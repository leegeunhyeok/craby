#import "NativeCalculatorCppProvider.h"
#import <ReactCommon/CallInvoker.h>
#import <ReactCommon/TurboModule.h>
#import "NativeCalculatorCpp.h"

@implementation NativeCalculatorCppProvider

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:
    (const facebook::react::ObjCTurboModule::InitParams &)params
{
  return std::make_shared<facebook::react::NativeCalculatorCpp>(params.jsInvoker);
}

@end
