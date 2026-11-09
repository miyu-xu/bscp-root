
#include <cassert>
#include <cerrno>
#include <cstdint>
#include <memory>
#include <optional>
#include <unordered_map>
#include <iostream>
#include <mutex>
#include <thread>
#include <vector>
using DescriptorType=int;
struct Descriptor { std::optional<int> value; int* closes; std::optional<int> release(){auto r=value; value.reset();return r;} ~Descriptor(){if(value)++*closes;} };
struct SyncDescriptorInfo { Descriptor descriptor; uint32_t streamHandleType; };
struct stream_renderer_handle { int64_t os_handle; uint32_t handle_type; };
class VirtioGpuFrontend {public: std::unordered_map<uint64_t,std::shared_ptr<SyncDescriptorInfo>> mSyncMap; int exportFence(uint64_t,stream_renderer_handle*);};
int VirtioGpuFrontend::exportFence(uint64_t fenceId, struct stream_renderer_handle* handle) {
    auto it = mSyncMap.find(fenceId);
    if (it == mSyncMap.end()) {
        return -EINVAL;
    }

    auto& entry = it->second;
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

    // The descriptor has transferred to the caller. Retaining the consumed
    // entry grows the map on every export and cannot support another export.
    mSyncMap.erase(it);
    return 0;
}

int main(){
 VirtioGpuFrontend v; int closes=0; stream_renderer_handle h{77,88};
 assert(v.exportFence(1,&h)==-EINVAL && h.os_handle==77);
 for(int i=1;i<=10000;++i){
  auto e=std::make_shared<SyncDescriptorInfo>(); e->descriptor.value=i;e->descriptor.closes=&closes;e->streamHandleType=0x20;v.mSyncMap[i]=e;e.reset();
  assert(v.exportFence(i,&h)==0 && h.os_handle==i && h.handle_type==0x20);
  assert(v.mSyncMap.empty() && closes==0);
  assert(v.exportFence(i,&h)==-EINVAL);
 }
 auto e=std::make_shared<SyncDescriptorInfo>();e->descriptor.closes=&closes;v.mSyncMap[2]=e;
 assert(v.exportFence(2,&h)==-EINVAL && v.mSyncMap.size()==1);

 v.mSyncMap.clear();e.reset();
 { auto pending=std::make_shared<SyncDescriptorInfo>();pending->descriptor.value=123;pending->descriptor.closes=&closes;v.mSyncMap[99]=pending; }
 v.mSyncMap.clear();assert(closes==1);
 std::mutex owner;
 std::vector<std::thread> workers;
 for(int t=0;t<4;++t) workers.emplace_back([&,t]{for(int j=0;j<1000;++j){
   std::lock_guard<std::mutex> lock(owner);auto id=100+t*1000+j;
   auto x=std::make_shared<SyncDescriptorInfo>();x->descriptor.value=id;x->descriptor.closes=&closes;x->streamHandleType=0x20;
   v.mSyncMap[id]=x;x.reset();stream_renderer_handle out{};
   assert(v.exportFence(id,&out)==0 && out.os_handle==id);assert(v.mSyncMap.empty());
 }});
 for(auto& w:workers)w.join();assert(closes==1);
 std::cout<<"PASS exact exportFence: missing, 10000 transfers, no premature close, bounded map, duplicate export, invalid descriptor\n";
}
