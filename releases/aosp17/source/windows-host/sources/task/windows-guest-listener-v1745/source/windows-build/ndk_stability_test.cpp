#include <binder/Common.h>
#include <android/binder_ibinder.h>
#include <android/binder_stability.h>
#include <cstdio>
int main(){
 auto cls=AIBinder_Class_define("bscp.test.Stability",[](void*)->void*{return nullptr;},[](void*){},[](AIBinder*,transaction_code_t,const AParcel*,AParcel*)->binder_status_t{return STATUS_UNKNOWN_TRANSACTION;});
 if(!cls)return 1;
 AIBinder* object=AIBinder_new(cls,nullptr);if(!object)return 2;
 AIBinder_markSystemStability(object);
 bool valid=AIBinder_isSystemStable(object)&&!AIBinder_isVendorStable(object)&&!AIBinder_requiresVintfDeclaration(object);
 AIBinder_decStrong(object);
 if(!valid)return 3;
 std::puts("REAL_A17_NDK_SYSTEM_STABILITY_PASS");return 0;
}
