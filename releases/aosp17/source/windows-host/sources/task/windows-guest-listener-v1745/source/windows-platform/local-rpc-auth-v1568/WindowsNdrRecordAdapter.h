#pragma once
#include "WindowsLocalRpcBridge.h"
#include <memory>
#include <vector>
#include <span>
namespace bscp::windows {
struct CallerIdentity;
class WindowsNdrRequestSnapshot;
DWORD importAuthenticatedWindowsNdrSnapshot(
    const std::shared_ptr<const WindowsNdrRequestSnapshot>&,
    const CallerIdentity&, CallerIdentity*);
class WindowsNdrReceivedRecord;
// Cannot be constructed from a token/PID/view supplied by a caller.
class WindowsNdrRequestSnapshot final {
    HANDLE token_=nullptr, process_=nullptr;
    DWORD pid_=0;
    std::vector<unsigned char> sid_;
    LUID logon_{};
    WindowsNdrRequestSnapshot()=default;
    friend class WindowsNdrReceivedRecord;
    friend DWORD importAuthenticatedWindowsNdrSnapshot(
        const std::shared_ptr<const WindowsNdrRequestSnapshot>&,
        const CallerIdentity&, CallerIdentity*);
public:
    ~WindowsNdrRequestSnapshot();
    WindowsNdrRequestSnapshot(const WindowsNdrRequestSnapshot&)=delete;
    WindowsNdrRequestSnapshot& operator=(const WindowsNdrRequestSnapshot&)=delete;
    DWORD checkedCallingPid(DWORD*) const;
    DWORD matchesOriginalProcess(HANDLE expected,bool* same) const;
    DWORD matchesOriginalSnapshot(const WindowsNdrRequestSnapshot& other,bool* same) const;
};
class WindowsNdrReceivedRecord final {
    struct Release { void operator()(WlrRecord* r)const {WlrRecordRelease(r);} };
    std::unique_ptr<WlrRecord,Release> record_;
    WlrView view_{};
    size_t consumed_=0;
    bool sealed_=false;
    explicit WindowsNdrReceivedRecord(std::unique_ptr<WlrRecord,Release>,const WlrView&);
public:
    // Sole factory: no adoption of arbitrary raw record/view or identity HANDLEs.
    static DWORD tryPop(uint64_t generation,uint64_t channel,
                        std::shared_ptr<WindowsNdrReceivedRecord>*);
    static DWORD tryPopAccepted(WlrAcceptedChannel*,std::shared_ptr<WindowsNdrReceivedRecord>*);
    WindowsNdrReceivedRecord(const WindowsNdrReceivedRecord&)=delete;
    WindowsNdrReceivedRecord& operator=(const WindowsNdrReceivedRecord&)=delete;
    uint64_t sequence()const{return view_.sequence;}
    size_t byteCount()const{return view_.bytes;}
    DWORD readExact(std::span<unsigned char>);
    // Root must validate its actual protocol header/body before sealing.
    // This enforces exact consumption of ONE immutable record, not parsing.
    DWORD sealConsumedRecord();
    DWORD makeSnapshot(std::shared_ptr<const WindowsNdrRequestSnapshot>*)const;
    std::span<const HANDLE> files()const{return {view_.files,view_.fileCount};}
    std::span<const HANDLE> pipes()const{return {view_.pipes,view_.pipeCount};}
};
#ifdef BSCP_WLR_ISOLATED_COMPONENT
// Inject failure in actual shared_ptr control-block allocator: stage1 record,
// stage2 authenticated snapshot. Does not alter the bridge or general allocator.
void windowsNdrTestFailControlBlock(unsigned stage);
#endif
}
