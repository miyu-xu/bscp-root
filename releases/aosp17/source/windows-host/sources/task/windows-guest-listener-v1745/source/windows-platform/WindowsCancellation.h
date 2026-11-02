#pragma once
#include <windows.h>
#include <memory>
#ifdef NO_ERROR
#undef NO_ERROR
#endif
namespace bscp::windows {
// Each waiter retains the original event owner. No close-while-waiting or handle reuse.
class Cancellation final {
    struct Event {
        HANDLE handle = CreateEventW(nullptr, TRUE, FALSE, nullptr);
        ~Event() { if (handle) CloseHandle(handle); }
    };
    std::shared_ptr<Event> event_;
public:
    enum class Outcome { ready, cancelled, timeout, error };
    struct Result { Outcome outcome; DWORD error; };
    Cancellation() : event_(std::make_shared<Event>()) {}
    bool valid() const { return event_ && event_->handle; }
    bool trigger() const { return valid() && SetEvent(event_->handle); }
    Result state() const {
        auto event=event_;
        if (!event || !event->handle) return {Outcome::error, ERROR_INVALID_HANDLE};
        DWORD r=WaitForSingleObject(event->handle,0);
        if(r==WAIT_OBJECT_0) return {Outcome::cancelled,0};
        if(r==WAIT_TIMEOUT) return {Outcome::ready,0};
        return {Outcome::error,r==WAIT_FAILED?GetLastError():ERROR_INVALID_DATA};
    }
    // The caller owns operationCompletion for the duration. It must not close or reset it
    // while waiting. On cancellation, pending I/O still requires CancelIoEx + completion drain.
    Result wait(HANDLE operationCompletion,DWORD timeout) const {
        auto event=event_;
        if(!event || !event->handle || !operationCompletion || operationCompletion==INVALID_HANDLE_VALUE)
            return {Outcome::error,ERROR_INVALID_HANDLE};
        HANDLE handles[]={event->handle,operationCompletion};
        DWORD r=WaitForMultipleObjects(2,handles,FALSE,timeout);
        if(r==WAIT_OBJECT_0) return {Outcome::cancelled,0};
        if(r==WAIT_OBJECT_0+1) return {Outcome::ready,0};
        if(r==WAIT_TIMEOUT) return {Outcome::timeout,0};
        return {Outcome::error,r==WAIT_FAILED?GetLastError():ERROR_INVALID_DATA};
    }
};
}
