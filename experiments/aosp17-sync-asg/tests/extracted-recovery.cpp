#include <vector>
#include <optional>
#include <cstdio>
#include <cassert>
#include <cerrno>
#include <chrono>
#include <thread>
#define VK_ERROR_UNKNOWN -13
#define mesa_loge(...) ((void)0)
#define VK_USE_PLATFORM_ANDROID_KHR 1
#define GFXSTREAM_SYNC_DEBUG 0
#define VK_SUCCESS 0
using VkResult=int;
struct Sync {int ret=0,err=0,waits=0,closes=0;int wait(int fd,int ms){assert(fd>=0&&ms>=0&&ms<=3000);++waits;if(ret==-2)std::this_thread::sleep_for(std::chrono::milliseconds(10));if(ret!=-2 && err==EINTR && waits==2){errno=0;return 0;}errno=err;return ret;} void close(int fd){assert(fd>=0);++closes;}};
Sync syncHelper;Sync* mSyncHelper=&syncHelper;int submits=0,flushes=0;
void flushStagingStreams(int,int,int,const int*){++flushes;}
int vkQueueSubmitEnc(int,int,int,int*,int){++submits;return 0;}
int extracted(std::vector<int> preSignalSyncFds){int context=0,enc=0,queue=0,submitCount=1,fence=0,input_result=0;std::vector<int> prunedSubmitInfos(1);const int* pSubmits=nullptr;
#if defined(VK_USE_PLATFORM_ANDROID_KHR) || DETECT_OS_LINUX
    for (auto fd : preSignalSyncFds) {
        // https://docs.vulkan.org/refpages/latest/refpages/source/VkImportSemaphoreFdInfoKHR.html
        // fd == -1 is treated as already signaled
        if (fd != -1) {
            // The imported descriptor remains owned by the semaphore. Waiting
            // neither consumes nor closes it. Do not submit a pruned dependency
            // after timeout/error, and do not report a fabricated device loss.
            const auto deadline = std::chrono::steady_clock::now() +
                                  std::chrono::milliseconds(3000);
            int waitResult;
            int waitError;
            do {
                const auto now = std::chrono::steady_clock::now();
                if (now >= deadline) return VK_ERROR_UNKNOWN;
                const auto remaining = std::chrono::duration_cast<std::chrono::milliseconds>(
                    deadline - now).count();
                waitResult = mSyncHelper->wait(fd, static_cast<int>(remaining));
                waitError = errno;
            } while (waitResult != 0 && waitError == EINTR);
            if (waitResult != 0) {
                mesa_loge("External semaphore dependency wait failed: errno=%d", waitError);
                return VK_ERROR_UNKNOWN;
            }
#if GFXSTREAM_SYNC_DEBUG
            mSyncHelper->debugPrint(fd);
#endif
        }
    }
#endif

    flushStagingStreams(context, queue, submitCount, pSubmits);

    // The main queue submission
    input_result = vkQueueSubmitEnc(enc, queue, submitCount, prunedSubmitInfos.data(), fence);
    if (input_result != VK_SUCCESS) return input_result;

return input_result;}
void destroy(std::optional<int>& fd){struct {std::optional<int> syncFd;} semInfo{fd};
#if defined(VK_USE_PLATFORM_ANDROID_KHR) || DETECT_OS_LINUX
    if (semInfo.syncFd.value_or(-1) >= 0) {
        mSyncHelper->close(semInfo.syncFd.value());
    }
#endif


fd.reset();}
int main(){
struct Case{const char* name;int fd,ret,err;};
Case cases[]={{"already_signaled_minus1",-1,0,0},{"signaled",41,0,0},{"timeout",42,-1,62},{"io_error",43,-1,EIO},{"interrupted",44,-1,EINTR},{"persistent_interruption",45,-2,EINTR}};
for(auto c:cases){syncHelper={};syncHelper.ret=c.ret;syncHelper.err=c.err;submits=0;flushes=0;std::optional<int> fd=c.fd;
auto started=std::chrono::steady_clock::now();int result=extracted({c.fd});if(c.ret==-2){auto elapsed=std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now()-started).count();assert(elapsed>=2900&&elapsed<4000);}int close_before=syncHelper.closes;bool retained=fd.has_value();destroy(fd);
bool success=c.ret==0||(c.err==EINTR&&c.ret!=-2);assert(result==(success?0:-13));assert(submits==(success?1:0));assert(flushes==(success?1:0));assert(close_before==0&&retained);assert(syncHelper.closes==(c.fd>=0?1:0));
std::printf("{\"case\":\"%s\",\"simulated_wait_return\":%d,\"submit_calls\":%d,\"result\":%d,\"flushes\":%d,\"closed_before_destroy\":%d,\"close_on_destroy\":%d,\"dependency_safety\":\"%s\"}\n",c.name,c.ret,submits,result,flushes,close_before,syncHelper.closes,"PASS");}
syncHelper={};syncHelper.ret=-1;syncHelper.err=EIO;submits=0;flushes=0;std::optional<int> retained=46;
assert(extracted({46})==-13 && submits==0 && flushes==0 && syncHelper.closes==0);
syncHelper.ret=0;syncHelper.err=0;assert(extracted({46})==0 && submits==1 && flushes==1);destroy(retained);assert(syncHelper.closes==1);
std::puts("{\"case\":\"retry_after_wait_error\",\"result\":0,\"total_submits\":1,\"close_on_destroy\":1,\"status\":\"PASS\"}");
}

