use std::path::PathBuf;

use craby_common::{
    constants::{
        android_path, android_src_main_path, dest_lib_name, java_base_path, jni_base_path,
    },
    utils::string::{flat_case, kebab_case, pascal_case, SanitizedString},
};
use indoc::formatdoc;

use crate::{constants::cxx_mod_cls_name, types::CodegenContext, utils::indent_str};

use super::types::{GenerateResult, Generator, GeneratorInvoker, Template};

pub struct AndroidTemplate;
pub struct AndroidGenerator;

pub enum AndroidFileType {
    JNIEntry,
    CmakeLists,
    ManifestXml,
    BuildGradle,
    GradleProps,
    RctPackage,
}

impl AndroidTemplate {
    fn file_path(&self, file_type: &AndroidFileType, project_name: &str) -> PathBuf {
        match file_type {
            AndroidFileType::JNIEntry => PathBuf::from("OnLoad.cpp"),
            AndroidFileType::CmakeLists => PathBuf::from("CMakeLists.txt"),
            AndroidFileType::ManifestXml => PathBuf::from("AndroidManifest.xml"),
            AndroidFileType::BuildGradle => PathBuf::from("build.gradle"),
            AndroidFileType::GradleProps => PathBuf::from("gradle.properties"),
            AndroidFileType::RctPackage => {
                PathBuf::from(format!("{}Package.kt", pascal_case(project_name)))
            }
        }
    }

    /// Returns `JNI_OnLoad` function implementation
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// jint JNI_OnLoad(JavaVM *vm, void *reserved) {
    ///   facebook::react::registerCxxModuleToGlobalModuleMap(
    ///     craby::mymodule::MyTestModule::kModuleName,
    ///     [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {
    ///       return std::make_shared<craby::mymodule::MyTestModule>(jsInvoker);
    ///     });
    ///   return JNI_VERSION_1_6;
    /// }
    ///
    /// extern "C"
    /// JNIEXPORT void JNICALL
    /// Java_com_mymodule_MyTestModulePackage_nativeSetDataPath(JNIEnv *env, jclass clazz, jstring jDataPath) {
    ///     auto dataPath = std::string(env->GetStringUTFChars(jDataPath, nullptr));
    ///     craby::mymodule::MyTestModule::dataPath = dataPath;
    /// }
    /// ```
    fn jni_entry(&self, ctx: &CodegenContext) -> Result<String, anyhow::Error> {
        let mut cxx_includes = vec![];
        let mut cxx_prepares = Vec::with_capacity(ctx.schemas.len());
        let mut cxx_registers = Vec::with_capacity(ctx.schemas.len());
        let jni_extern_fn_name = ctx
            .android_package_name
            .split('.')
            .map(flat_case)
            .collect::<Vec<_>>()
            .join("_");

        let jni_fn_name = format!(
            "Java_{}_{}Package_nativeSetDataPath",
            jni_extern_fn_name,
            pascal_case(&ctx.name)
        );

        for schema in &ctx.schemas {
            let cxx_mod = cxx_mod_cls_name(&schema.module_name);
            let flat_name = flat_case(&schema.module_name);

            let cxx_namespace = format!("craby::{}::{}", flat_name, cxx_mod);
            let cxx_include = format!("#include <{cxx_mod}.hpp>");
            let cxx_prepare = format!("{cxx_namespace}::dataPath = dataPath;");
            let cxx_register = formatdoc! {
                r#"
                facebook::react::registerCxxModuleToGlobalModuleMap(
                  {cxx_namespace}::kModuleName,
                  [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {{
                    return std::make_shared<{cxx_namespace}>(jsInvoker);
                  }});"#,
                cxx_namespace = cxx_namespace
            };

            cxx_includes.push(cxx_include);
            cxx_prepares.push(cxx_prepare);
            cxx_registers.push(cxx_register);
        }

        let content = formatdoc! {
            r#"
            {cxx_includes}
            #include <ReactCommon/CxxTurboModuleUtils.h>
            #include <jni.h>

            jint JNI_OnLoad(JavaVM *vm, void *reserved) {{
            {cxx_registers}
              return JNI_VERSION_1_6;
            }}
            
            extern "C"
            JNIEXPORT void JNICALL
            {jni_fn_name}(JNIEnv *env, jclass clazz, jstring jDataPath) {{
              auto dataPath = std::string(env->GetStringUTFChars(jDataPath, nullptr));
            {cxx_prepares}
            }}"#,
            cxx_includes = cxx_includes.join("\n"),
            cxx_prepares = indent_str(&cxx_prepares.join("\n"), 2),
            cxx_registers = indent_str(&cxx_registers.join("\n"), 2),
        };

        Ok(content)
    }

    /// Generates the Android.manifest.
    fn manifest_xml(&self, ctx: &CodegenContext) -> String {
        formatdoc! {
            r#"
            <manifest xmlns:android="http://schemas.android.com/apk/res/android"
              package="{package_name}">
            </manifest>"#,
            package_name = ctx.android_package_name,
        }
    }

    /// Generates the build.gradle.
    fn build_gradle(&self, ctx: &CodegenContext) -> String {
        formatdoc! {
            r#"
            def reactNativeArchitectures() {{
              def value = rootProject.getProperties().get("reactNativeArchitectures")
              return value ? value.split(",") : ["armeabi-v7a", "x86", "x86_64", "arm64-v8a"]
            }}

            buildscript {{
              ext.getExtOrDefault = {{name ->
                return rootProject.ext.has(name) ? rootProject.ext.get(name) : project.properties['{pascal_name}_' + name]
              }}

              repositories {{
                google()
                mavenCentral()
              }}

              dependencies {{
                classpath "com.android.tools.build:gradle:8.7.2"
                // noinspection DifferentKotlinGradleVersion
                classpath "org.jetbrains.kotlin:kotlin-gradle-plugin:${{getExtOrDefault('kotlinVersion')}}"
              }}
            }}

            apply plugin: "com.android.library"
            apply plugin: "kotlin-android"
            apply plugin: "com.facebook.react"

            def getExtOrIntegerDefault(name) {{
              return rootProject.ext.has(name) ? rootProject.ext.get(name) : (project.properties["{pascal_name}_" + name]).toInteger()
            }}

            android {{
              namespace "{package_name}"

              compileSdkVersion getExtOrIntegerDefault("compileSdkVersion")

              defaultConfig {{
                minSdkVersion getExtOrIntegerDefault("minSdkVersion")
                targetSdkVersion getExtOrIntegerDefault("targetSdkVersion")

                externalNativeBuild {{
                  cmake {{
                    targets "cxx-{kebab_name}"
                    cppFlags "-frtti -fexceptions -Wall -Wextra -fstack-protector-all"
                    arguments "-DANDROID_STL=c++_shared", "-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON"
                    abiFilters (*reactNativeArchitectures())
                    buildTypes {{
                      debug {{
                        cppFlags "-O1 -g"
                      }}
                      release {{
                        cppFlags "-O2"
                      }}
                    }}
                  }}
                }}
              }}

              externalNativeBuild {{
                cmake {{
                  path "CMakeLists.txt"
                }}
              }}

              buildFeatures {{
                buildConfig true
                prefab true
              }}

              buildTypes {{
                debug {{
                  jniDebuggable true
                }}
                release {{
                  minifyEnabled false
                  externalNativeBuild {{
                    cmake {{
                      arguments "-DCMAKE_BUILD_TYPE=Release"
                    }}
                  }}
                }}
              }}

              lintOptions {{
                disable "GradleCompatible"
              }}

              compileOptions {{
                sourceCompatibility JavaVersion.VERSION_1_8
                targetCompatibility JavaVersion.VERSION_1_8
              }}
            }}

            repositories {{
              mavenCentral()
              google()
            }}

            def kotlin_version = getExtOrDefault("kotlinVersion")

            dependencies {{
              implementation "com.facebook.react:react-android"
              implementation "com.facebook.react:hermes-engine"
              implementation "org.jetbrains.kotlin:kotlin-stdlib:$kotlin_version"
            }}

            react {{
              jsRootDir = file("../src/")
              libraryName = "{pascal_name}_stub"
              codegenJavaPackageName = "{package_name}"
            }}"#,
            pascal_name = pascal_case(&ctx.name),
            kebab_name = kebab_case(&ctx.name),
            package_name = ctx.android_package_name,
        }
    }

    /// Generates the gradle.properties.
    fn grable_props(&self, ctx: &CodegenContext) -> String {
        formatdoc! {
            r#"
            {pascal_name}_kotlinVersion=2.0.21
            {pascal_name}_minSdkVersion=24
            {pascal_name}_targetSdkVersion=34
            {pascal_name}_compileSdkVersion=35
            {pascal_name}_ndkVersion=27.1.12297006"#,
            pascal_name = pascal_case(&ctx.name)
        }
    }

    /// Generates the CMakeLists.txt for Android native module build configuration.
    ///
    /// # Generated Code
    ///
    /// ```cmake
    /// cmake_minimum_required(VERSION 3.13)
    ///
    /// project(craby-my-app)
    ///
    /// set (CMAKE_VERBOSE_MAKEFILE ON)
    /// set (CMAKE_CXX_STANDARD 20)
    ///
    /// find_package(ReactAndroid REQUIRED CONFIG)
    ///
    /// # Import the pre-built Craby library
    /// add_library(my-app-lib STATIC IMPORTED)
    /// set_target_properties(my-app-lib PROPERTIES
    ///   IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/src/main/jni/libs/${ANDROID_ABI}/libcraby_my_app.a"
    /// )
    /// target_include_directories(my-app-lib INTERFACE
    ///   "${CMAKE_SOURCE_DIR}/src/main/jni/include"
    /// )
    ///
    /// # Generated C++ source files by Craby
    /// add_library(cxx-my-app SHARED
    ///   src/main/jni/OnLoad.cpp
    ///   src/main/jni/src/ffi.rs.cc
    ///   ../cpp/CxxMyTestModule.cpp
    /// )
    /// target_include_directories(cxx-my-app PRIVATE
    ///   ../cpp
    /// )
    ///
    /// target_link_libraries(cxx-my-app
    ///   # android
    ///   ReactAndroid::reactnative
    ///   ReactAndroid::jsi
    ///   # my-app-lib
    ///   my-app-lib
    /// )
    ///
    /// # From ReactAndroid/cmake-utils/folly-flags.cmake
    /// target_compile_definitions(cxx-craby-test PRIVATE
    ///   -DFOLLY_NO_CONFIG=1
    ///   -DFOLLY_HAVE_CLOCK_GETTIME=1
    ///   -DFOLLY_USE_LIBCPP=1
    ///   -DFOLLY_CFG_NO_COROUTINES=1
    ///   -DFOLLY_MOBILE=1
    ///   -DFOLLY_HAVE_RECVMMSG=1
    ///   -DFOLLY_HAVE_PTHREAD=1
    ///   # Once we target android-23 above, we can comment
    ///   # the following line. NDK uses GNU style stderror_r() after API 23.
    ///   -DFOLLY_HAVE_XSI_STRERROR_R=1
    /// )
    /// ```
    fn cmakelists(&self, ctx: &CodegenContext) -> String {
        let kebab_name = kebab_case(&ctx.name);
        let lib_name = dest_lib_name(&SanitizedString::from(&ctx.name));
        let cxx_mod_cpp_files = ctx
            .schemas
            .iter()
            .map(|schema| format!("../cpp/{}.cpp", cxx_mod_cls_name(&schema.module_name)))
            .collect::<Vec<_>>();

        formatdoc! {
            r#"
            cmake_minimum_required(VERSION 3.13)

            project(craby-{kebab_name})

            set (CMAKE_VERBOSE_MAKEFILE ON)
            set (CMAKE_CXX_STANDARD 20)

            find_package(ReactAndroid REQUIRED CONFIG)

            # Import the pre-built Craby library
            add_library({kebab_name}-lib STATIC IMPORTED)
            set_target_properties({kebab_name}-lib PROPERTIES
              IMPORTED_LOCATION "${{CMAKE_SOURCE_DIR}}/src/main/jni/libs/${{ANDROID_ABI}}/{lib_name}"
            )
            target_include_directories({kebab_name}-lib INTERFACE
              "${{CMAKE_SOURCE_DIR}}/src/main/jni/include"
            )

            # Generated C++ source files by Craby
            add_library(cxx-{kebab_name} SHARED
              src/main/jni/OnLoad.cpp
              src/main/jni/src/ffi.rs.cc
            {cxx_mod_cpp_files}
            )
            target_include_directories(cxx-{kebab_name} PRIVATE
              ../cpp
            )

            target_link_libraries(cxx-{kebab_name}
              # android
              ReactAndroid::reactnative
              ReactAndroid::jsi
              # {kebab_name}-lib
              {kebab_name}-lib
            )

            # From ReactAndroid/cmake-utils/folly-flags.cmake
            target_compile_definitions(cxx-craby-test PRIVATE
              -DFOLLY_NO_CONFIG=1
              -DFOLLY_HAVE_CLOCK_GETTIME=1
              -DFOLLY_USE_LIBCPP=1
              -DFOLLY_CFG_NO_COROUTINES=1
              -DFOLLY_MOBILE=1
              -DFOLLY_HAVE_RECVMMSG=1
              -DFOLLY_HAVE_PTHREAD=1
              # Once we target android-23 above, we can comment
              # the following line. NDK uses GNU style stderror_r() after API 23.
              -DFOLLY_HAVE_XSI_STRERROR_R=1
            )"#,
            kebab_name = kebab_name,
            lib_name = lib_name,
            cxx_mod_cpp_files = indent_str(&cxx_mod_cpp_files.join("\n"), 2),
        }
    }

    fn rct_package(&self, ctx: &CodegenContext) -> String {
        let lib_name = format!("cxx-{}", kebab_case(&ctx.name));
        let pascal_name = pascal_case(&ctx.name);
        let jni_prepare_module_names = ctx
            .schemas
            .iter()
            .map(|schema| format!("\"__craby{}_JNI_prepare__\"", schema.module_name))
            .collect::<Vec<_>>();

        formatdoc! {
            r#"
            package {package_name}

            import com.facebook.react.BaseReactPackage
            import com.facebook.react.bridge.NativeModule
            import com.facebook.react.bridge.ReactApplicationContext
            import com.facebook.react.bridge.ReactContextBaseJavaModule
            import com.facebook.react.module.model.ReactModuleInfo
            import com.facebook.react.module.model.ReactModuleInfoProvider
            import com.facebook.react.turbomodule.core.interfaces.TurboModule
            import com.facebook.soloader.SoLoader
            import javax.annotation.Nonnull

            class {pascal_name}Package : BaseReactPackage() {{
              companion object {{
                val JNI_PREPARE_MODULE_NAME = setOf(
            {jni_prepare_module_names}
                )
              }}

              init {{
                SoLoader.loadLibrary("{lib_name}")
              }}

              override fun getModule(name: String, reactContext: ReactApplicationContext): NativeModule? {{
                if (name in JNI_PREPARE_MODULE_NAME) {{
                  nativeSetDataPath(reactContext.filesDir.absolutePath)
                  return {pascal_name}Package.TurboModulePlaceholder(reactContext, name)
                }}
                return null
              }}

              override fun getReactModuleInfoProvider(): ReactModuleInfoProvider {{
                return ReactModuleInfoProvider {{
                  val moduleInfos: MutableMap<String, ReactModuleInfo> = HashMap()
                  JNI_PREPARE_MODULE_NAME.forEach {{ name ->
                    moduleInfos[name] = ReactModuleInfo(
                      name,
                      name,
                      false,  // canOverrideExistingModule
                      false,  // needsEagerInit
                      false,  // isCxxModule
                      true,  // isTurboModule
                    )
                  }}
                  moduleInfos
                }}
              }}

              private external fun nativeSetDataPath(dataPath: String)

              class TurboModulePlaceholder(reactContext: ReactApplicationContext?, private val name: String) :
                ReactContextBaseJavaModule(reactContext),
                TurboModule {{
                @Nonnull
                override fun getName(): String {{
                  return name
                }}
              }}
            }}"#,
            package_name = ctx.android_package_name,
            lib_name = lib_name,
            pascal_name = pascal_name,
            jni_prepare_module_names = indent_str(&jni_prepare_module_names.join(",\n"), 6),
        }
    }
}

impl Template for AndroidTemplate {
    type FileType = AndroidFileType;

    fn render(
        &self,
        ctx: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let path = self.file_path(file_type, &ctx.name);
        let content = match file_type {
            AndroidFileType::JNIEntry => self.jni_entry(ctx),
            AndroidFileType::CmakeLists => Ok(self.cmakelists(ctx)),
            AndroidFileType::ManifestXml => Ok(self.manifest_xml(ctx)),
            AndroidFileType::BuildGradle => Ok(self.build_gradle(ctx)),
            AndroidFileType::GradleProps => Ok(self.grable_props(ctx)),
            AndroidFileType::RctPackage => Ok(self.rct_package(ctx)),
        }?;

        Ok(vec![(path, content)])
    }
}

impl Default for AndroidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AndroidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Generator<AndroidTemplate> for AndroidGenerator {
    fn cleanup(_: &CodegenContext) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn generate(&self, ctx: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let android_base_path = android_path(&ctx.root);
        let android_src_path = android_src_main_path(&ctx.root);
        let jni_base_path = jni_base_path(&ctx.root);
        let java_base_path = java_base_path(&ctx.root, &ctx.android_package_name);
        let template = self.template_ref();
        let mut files = vec![];

        let jni_res = template
            .render(ctx, &AndroidFileType::JNIEntry)?
            .into_iter()
            .map(|(path, content)| GenerateResult {
                path: jni_base_path.join(path),
                content,
                overwrite: true,
            })
            .collect::<Vec<_>>();

        let android_base_path_targets = [
            AndroidFileType::CmakeLists,
            AndroidFileType::BuildGradle,
            AndroidFileType::GradleProps,
        ];

        for target in &android_base_path_targets {
            let res = template
                .render(ctx, target)?
                .into_iter()
                .map(|(path, content)| GenerateResult {
                    path: android_base_path.join(path),
                    content,
                    overwrite: true,
                })
                .collect::<Vec<_>>();

            files.extend(res);
        }

        let manifest_xml_res = template
            .render(ctx, &AndroidFileType::ManifestXml)?
            .into_iter()
            .map(|(path, content)| GenerateResult {
                path: android_src_path.join(path),
                content,
                overwrite: true,
            })
            .collect::<Vec<_>>();

        let rct_package_res = template
            .render(ctx, &AndroidFileType::RctPackage)?
            .into_iter()
            .map(|(path, content)| GenerateResult {
                path: java_base_path.join(path),
                content,
                overwrite: true,
            })
            .collect::<Vec<_>>();

        files.extend(jni_res);
        files.extend(manifest_xml_res);
        files.extend(rct_package_res);

        Ok(files)
    }

    fn template_ref(&self) -> &AndroidTemplate {
        &AndroidTemplate
    }
}

impl GeneratorInvoker for AndroidGenerator {
    fn invoke_generate(&self, ctx: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
        self.generate(ctx)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::tests::get_codegen_context;

    use super::*;

    #[test]
    fn test_android_generator() {
        let ctx = get_codegen_context();
        let generator = AndroidGenerator::new();
        let results = generator.generate(&ctx).unwrap();
        let result = results
            .iter()
            .map(|res| format!("{}\n{}", res.path.display(), res.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        assert_snapshot!(result);
    }
}
