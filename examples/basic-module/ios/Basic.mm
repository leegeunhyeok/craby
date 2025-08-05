#import "Basic.h"
#import "libbasicmodule.h"

@implementation Basic
RCT_EXPORT_MODULE()

- (NSNumber *)numericMethod:(double)arg {
    NSNumber *result = @(numericMethod(arg));

    return result;
}

- (NSString *)stringMethod:(NSString *)arg {
    NSString *result = @(stringMethod([arg UTF8String]));

    return result;
}

- (NSNumber* )booleanMethod:(BOOL)arg {
    NSNumber* result = @(booleanMethod(arg ? true : false));

    return result;
}

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:
    (const facebook::react::ObjCTurboModule::InitParams &)params
{
    return std::make_shared<facebook::react::NativeBasicSpecJSI>(params);
}

@end
