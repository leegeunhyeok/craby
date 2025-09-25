use std::path::PathBuf;

use crate::{parser::native_spec_parser::try_parse_schema, types::CodegenContext};

pub fn get_codegen_context() -> CodegenContext {
    let schemas = try_parse_schema(
        "
        import type { TurboModule } from 'react-native';
        import { TurboModuleRegistry } from 'react-native';

        export interface TestObject {
            foo: string;
            bar: number;
            baz: boolean;
            sub: SubObject | null;
        }

        export type SubObject = {
            a: string | null;
            b: number;
            c: boolean;
        };

        export type MaybeNumber = number | null;

        export enum MyEnum {
            Foo = 'foo',
            Bar = 'bar',
            Baz = 'baz',
        }

        export enum SwitchState {
            Off = 0,
            On = 1,
        }

        export interface Spec extends TurboModule {
            numericMethod(arg: number): number;
            booleanMethod(arg: boolean): boolean;
            stringMethod(arg: string): string;
            objectMethod(arg: TestObject): TestObject;
            arrayMethod(arg: number[]): number[];
            enumMethod(arg0: MyEnum, arg1: SwitchState): string;
            nullableMethod(arg: number | null): MaybeNumber;
            promiseMethod(arg: number): Promise<number>;
        }

        export default TurboModuleRegistry.getEnforcing<Spec>('CrabyTest');
        ",
    )
    .unwrap();

    CodegenContext {
        name: "test_module".to_string(),
        root: PathBuf::from("."),
        schemas,
    }
}
