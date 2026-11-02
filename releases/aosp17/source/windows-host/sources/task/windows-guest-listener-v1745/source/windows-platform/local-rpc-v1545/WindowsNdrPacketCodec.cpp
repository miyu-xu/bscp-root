#include "WindowsNdrPacketCodec.h"
#include <array>
#include <algorithm>
#include <new>
namespace bscp::windows {
namespace {
constexpr unsigned char magic[4]={'W','L','R','1'};
void u32(unsigned char* p,uint32_t v){for(unsigned i=0;i<4;++i)p[i]=static_cast<unsigned char>(v>>(8*i));}
uint32_t read32(const unsigned char* p){uint32_t v=0;for(unsigned i=0;i<4;++i)v|=uint32_t(p[i])<<(8*i);return v;}
}
DWORD encodeWindowsNdrPacket(std::span<const unsigned char> body,std::span<const HANDLE> handles,WindowsNdrEncodedPacket* out){
 if(!out)return ERROR_INVALID_PARAMETER;*out={};if(body.size()>kNdrBinderPacketMax||handles.size()>kNdrDescriptorMax)return ERROR_INVALID_PARAMETER;
 try {WindowsNdrEncodedPacket encoded;const size_t prefix=16+2*handles.size();encoded.bytes.resize(prefix+body.size());
  for(size_t i=0;i<4;++i)encoded.bytes[i]=magic[i];encoded.bytes[4]=1;encoded.bytes[6]=static_cast<unsigned char>(handles.size());u32(encoded.bytes.data()+8,static_cast<uint32_t>(body.size()));
  for(size_t i=0;i<handles.size();++i){HANDLE h=handles[i];if(!h||h==INVALID_HANDLE_VALUE)return ERROR_INVALID_HANDLE;DWORD type=GetFileType(h);unsigned char kind,index;
   if(type==FILE_TYPE_DISK){kind=0;index=static_cast<unsigned char>(encoded.files.size());encoded.files.push_back(h);}
   else if(type==FILE_TYPE_PIPE){DWORD flags=0;if(!GetNamedPipeInfo(h,&flags,nullptr,nullptr,nullptr))return GetLastError();kind=1;index=static_cast<unsigned char>(encoded.pipes.size());encoded.pipes.push_back(h);}
   else return ERROR_NOT_SUPPORTED;
   encoded.bytes[16+2*i]=kind;encoded.bytes[17+2*i]=index;
  }
  std::copy(body.begin(),body.end(),encoded.bytes.begin()+prefix);*out=std::move(encoded);return ERROR_SUCCESS;
 }catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
}
DWORD decodeWindowsNdrPacket(const WlrView& v,WindowsNdrDecodedPacket* out){
 if(!out)return ERROR_INVALID_PARAMETER;*out={};if(!v.data||v.bytes<16||v.bytes>kNdrEnvelopeMax||v.fileCount>kNdrDescriptorMax||v.pipeCount>kNdrDescriptorMax||v.fileCount+v.pipeCount>kNdrDescriptorMax||(!v.files&&v.fileCount)||(!v.pipes&&v.pipeCount))return ERROR_INVALID_DATA;
 const unsigned char* p=v.data;for(size_t i=0;i<4;++i)if(p[i]!=magic[i])return ERROR_INVALID_DATA;
 if(p[4]!=1||p[5]||p[7]||read32(p+12))return ERROR_INVALID_DATA;const size_t count=p[6],body=read32(p+8),prefix=16+2*count;
 if(count!=size_t(v.fileCount)+v.pipeCount||body>kNdrBinderPacketMax||prefix+body!=v.bytes)return ERROR_INVALID_DATA;
 try {WindowsNdrDecodedPacket decoded;decoded.orderedHandles.reserve(count);std::array<bool,kNdrDescriptorMax> files{},pipes{};
  for(size_t i=0;i<count;++i){unsigned kind=p[16+2*i],index=p[17+2*i];HANDLE h;
   if(kind==0){if(index>=v.fileCount||files[index])return ERROR_INVALID_DATA;files[index]=true;h=v.files[index];}
   else if(kind==1){if(index>=v.pipeCount||pipes[index])return ERROR_INVALID_DATA;pipes[index]=true;h=v.pipes[index];}
   else return ERROR_INVALID_DATA;if(!h||h==INVALID_HANDLE_VALUE)return ERROR_INVALID_HANDLE;decoded.orderedHandles.push_back(h);
  }
  // Equal counts + unique in-range indices prove no typed resource is omitted.
  decoded.binderBytes={p+prefix,body};*out=std::move(decoded);return ERROR_SUCCESS;
 }catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
}
}
