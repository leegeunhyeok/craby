#import "CxxBasicModule.hpp"
#import <ReactCommon/CxxTurboModuleUtils.h>

@interface BasicModuleProvider : NSObject
@end

@implementation BasicModuleProvider
+ (void)load {
  facebook::react::registerCxxModuleToGlobalModuleMap(
      craby::basicmodule::CxxBasicModule::kModuleName,
      [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {
        return std::make_shared<craby::basicmodule::CxxBasicModule>(jsInvoker);
      });
}
@end
