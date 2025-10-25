require "json"

package = JSON.parse(File.read(File.join(__dir__, "package.json")))

Pod::Spec.new do |s|
  s.name         = "{{ pascal_name }}"
  s.version      = package["version"]
  s.summary      = package["description"]
  s.homepage     = package["homepage"]
  s.license      = package["license"]
  s.authors      = package["author"]

  s.platforms    = { :ios => min_ios_version_supported }
  s.source       = { :git => "{{ repository_url }}.git", :tag => "#{s.version}" }

  s.source_files = ["ios/**/*.{{{{raw}}}}{h,m,mm,cc,cpp}{{{{/raw}}}}", "cpp/**/*.{{{{raw}}}}{hpp,cpp}{{{{/raw}}}}"]
  s.private_header_files = "ios/include/*.h"
  s.vendored_frameworks = "ios/framework/lib{{ flat_name }}.xcframework"

  s.pod_target_xcconfig = {
    "CLANG_CXX_LANGUAGE_STANDARD" => "c++20",
  }

  install_modules_dependencies(s)
end
