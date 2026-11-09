
#include <cassert>
#include <cerrno>
#include <cstdint>
#include <memory>
#include <optional>
#include <map>
#include <mutex>
#include <thread>
#include <atomic>
#include <iostream>
#define GFXSTREAM_ERROR(...) ((void)0)
using DescriptorType=int;
std::atomic<int> closes{0};
struct Descriptor {std::optional<int> value;Descriptor()=default;Descriptor(Descriptor&& o):value(o.release()){} Descriptor& operator=(Descriptor&& o){if(value)++closes;value=o.release();return *this;}std::optional<int> release(){auto r=value;value.reset();return r;}~Descriptor(){if(value)++closes;}};
struct SyncDescriptorInfo{Descriptor descriptor;uint32_t streamHandleType=0x20;};
struct stream_renderer_handle{int64_t os_handle;uint32_t handle_type;};
struct Context{std::optional<SyncDescriptorInfo> pending;std::optional<SyncDescriptorInfo> TakeSync(){auto r=std::move(pending);pending.reset();return r;}};
class VirtioGpuFrontend{public:struct ContextSyncDescriptor{uint32_t contextId;std::shared_ptr<SyncDescriptorInfo> info;};std::map<uint32_t,Context> mContexts;std::map<uint64_t,ContextSyncDescriptor> mSyncMap;int acquireContextFence(uint32_t,uint64_t);int exportFence(uint64_t,stream_renderer_handle*);};
class ExternalObjectManager{public:std::mutex mMutex;std::map<std::pair<uint32_t,uint64_t>,SyncDescriptorInfo> mSyncDescriptorInfos;void removeSyncDescriptorsForContext(uint32_t);void add(uint32_t c,uint64_t s){std::lock_guard<std::mutex> l(mMutex);mSyncDescriptorInfos[{c,s}].descriptor.value=int(s);}};
int VirtioGpuFrontend::acquireContextFence(uint32_t contextId, uint64_t fenceId) {
    auto contextIt = mContexts.find(contextId);
    if (contextIt == mContexts.end()) {
        GFXSTREAM_ERROR("failed to acquire context %u fence: context not found", contextId);
        return -EINVAL;
    }
    auto& context = contextIt->second;

    if (mSyncMap.find(fenceId) != mSyncMap.end()) {
        return -EINVAL; // Do not consume pending sync or replace another owner.
    }
    auto syncInfoOpt = context.TakeSync();
    if (!syncInfoOpt) {
        GFXSTREAM_ERROR("failed to acquire context %u fence: no sync acquired", contextId);
        return -EINVAL;
    }

    mSyncMap.emplace(fenceId, ContextSyncDescriptor{contextId,
        std::make_shared<SyncDescriptorInfo>(std::move(*syncInfoOpt))});

    return 0;
}
int VirtioGpuFrontend::exportFence(uint64_t fenceId, struct stream_renderer_handle* handle) {
    auto it = mSyncMap.find(fenceId);
    if (it == mSyncMap.end()) {
        return -EINVAL;
    }

    auto& entry = it->second.info;
    DescriptorType rawDescriptor;
    auto rawDescriptorOpt = entry->descriptor.release();
    if (rawDescriptorOpt)
        rawDescriptor = *rawDescriptorOpt;
    else
        return -EINVAL;

    handle->handle_type = entry->streamHandleType;

#if 0
    handle->os_handle = static_cast<int64_t>(reinterpret_cast<intptr_t>(rawDescriptor));
#else
    handle->os_handle = static_cast<int64_t>(rawDescriptor);
#endif

    mSyncMap.erase(it); // Descriptor ownership was transferred to the caller.
    return 0;
}
void ExternalObjectManager::removeSyncDescriptorsForContext(uint32_t ctxId) {
    std::lock_guard<std::mutex> lock(mMutex);
    for (auto it = mSyncDescriptorInfos.begin(); it != mSyncDescriptorInfos.end();) {
        if (it->first.first == ctxId) {
            it = mSyncDescriptorInfos.erase(it);
        } else {
            ++it;
        }
    }
}
int main(){
 VirtioGpuFrontend v;stream_renderer_handle out{};
 assert(v.acquireContextFence(1,1)==-EINVAL);
 v.mContexts[1].pending.emplace();v.mContexts[1].pending->descriptor.value=41;
 assert(v.acquireContextFence(1,10)==0);
 v.mContexts[1].pending.emplace();v.mContexts[1].pending->descriptor.value=42;
 assert(v.acquireContextFence(1,10)==-EINVAL && v.mContexts[1].pending.has_value());
 assert(v.exportFence(10,&out)==0 && out.os_handle==41 && closes==0);
 assert(v.exportFence(10,&out)==-EINVAL);
 assert(v.acquireContextFence(1,11)==0);
 v.mSyncMap.clear();v.mContexts.clear();assert(closes==1);
 ExternalObjectManager m;
 for(int round=0;round<100;++round){
  std::thread a([&]{for(int i=0;i<100;++i)m.add(1,i);});
  std::thread b([&]{for(int i=0;i<100;++i)m.add(2,i);});a.join();
  m.removeSyncDescriptorsForContext(1);b.join();
  {std::lock_guard<std::mutex> l(m.mMutex);assert(m.mSyncDescriptorInfos.size()==100);for(auto& e:m.mSyncDescriptorInfos)assert(e.first.first==2);}
  m.removeSyncDescriptorsForContext(2);assert(m.mSyncDescriptorInfos.empty());
 }
 assert(closes==20001);
 std::cout<<"PASS duplicate ownership, missing context/export, transfer/destruction, 100 joined-producer retirement races with unrelated producer; 20000 exact closes\n";
}
