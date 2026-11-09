#include "gfxstream/synchronization/MessageChannel.h"
#include <future>
#include <chrono>
#include <cstdio>
#include <cstdlib>
using gfxstream::base::MessageChannel;
using namespace std::chrono_literals;
void need(bool x){if(!x)std::abort();}
int main(){
for(int i=0;i<100;i++){
 MessageChannel<int,4> q;for(int j=0;j<4;j++)need(q.trySend(j));need(!q.trySend(99));
 std::promise<void> started;auto ready=started.get_future();
 auto blocked=std::async(std::launch::async,[&]{started.set_value();return q.send(9);});ready.wait();
 need(blocked.wait_for(2ms)==std::future_status::timeout);q.stop();need(blocked.wait_for(1s)==std::future_status::ready);need(!blocked.get());int v=123;need(!q.receive(&v));need(v==123);need(!q.trySend(0));
 MessageChannel<int,4> empty;auto reader=std::async(std::launch::async,[&]{int cmd=5;return empty.receive(&cmd);});empty.stop();need(reader.wait_for(1s)==std::future_status::ready);need(!reader.get());
 MessageChannel<int,4> normal;need(normal.send(7));need(normal.receive(&v)&&v==7);normal.stop();
}
puts("PASS 100 full-queue sender cancellation, empty-reader wake, closed receive and normal FIFO");
}
