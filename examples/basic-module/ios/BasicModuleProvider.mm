#import <ReactCommon/CxxTurboModuleUtils.h>
#import "CxxBasicModule.hpp"

@interface Basic : NSObject
@end

@implementation Basic
+ (void)load {
  facebook::react::registerCxxModuleToGlobalModuleMap(
      craby::basicmodule::CxxBasicModule::kModuleName,
      [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {
        return std::make_shared<craby::basicmodule::CxxBasicModule>(
            jsInvoker);
      });
}
@end
