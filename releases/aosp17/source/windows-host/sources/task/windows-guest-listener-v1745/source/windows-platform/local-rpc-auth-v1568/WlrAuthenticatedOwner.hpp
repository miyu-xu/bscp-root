#pragma once
#include "WindowsLocalRpcBridge.h"
#include <utility>
class WlrInitialOwner final {
 WlrAuthenticatedOwner* value_=nullptr;
 public:
 WlrInitialOwner()=default;
 WlrInitialOwner(const WlrInitialOwner&)=delete;
 WlrInitialOwner& operator=(const WlrInitialOwner&)=delete;
 WlrInitialOwner(WlrInitialOwner&& rhs) noexcept:value_(std::exchange(rhs.value_,nullptr)){}
 WlrInitialOwner& operator=(WlrInitialOwner&&)=delete;
 ~WlrInitialOwner(){if(value_)(void)WlrAuthenticatedRelease(&value_);}
 DWORD create(const wchar_t* endpoint,const void* sid,DWORD bytes,HANDLE launch){if(value_)return ERROR_ALREADY_EXISTS;return WlrCreateInitialAuthenticated(endpoint,sid,bytes,launch,&value_);}
 DWORD close(){return value_?WlrAuthenticatedRelease(&value_):ERROR_SUCCESS;}
 DWORD check_live()const{return WlrAuthenticatedCheckLive(value_);}
 explicit operator bool()const noexcept{return value_!=nullptr;}
};
