#import "Calculator.h"
#import "libcrabycalculator.h"

@implementation Calculator
RCT_EXPORT_MODULE()

- (NSNumber *)add:(double)a b:(double)b {
    NSNumber *result = @(add(a, b));

    return result;
}

- (NSNumber *)subtract:(double)a b:(double)b {
    NSNumber *result = @(subtract(a, b));

    return result;
}

- (NSNumber *)multiply:(double)a b:(double)b {
    NSNumber *result = @(multiply(a, b));

    return result;
}

- (NSNumber *)divide:(double)a b:(double)b {
    NSNumber *result = @(divide(a, b));

    return result;
}

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:
    (const facebook::react::ObjCTurboModule::InitParams &)params
{
    return std::make_shared<facebook::react::NativeCalculatorSpecJSI>(params);
}

@end
