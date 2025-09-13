require "json"

package = JSON.parse(File.read(File.join(__dir__, "package.json")))

Pod::Spec.new do |s|
  s.name         = "CrabyTest"
  s.version      = package["version"]
  s.summary      = package["description"]
  s.homepage     = package["homepage"]
  s.license      = package["license"]
  s.authors      = package["author"]

  s.platforms    = { :ios => min_ios_version_supported }
  s.source       = { :git => "https://github.com/leegeunhyeok/craby.git", :tag => "#{s.version}" }

  s.source_files = ["ios/**/*.{h,m,mm,cc,cpp}", "cpp/**/*.{hpp,cpp}"]
  s.private_header_files = "ios/include/*.h"

  s.preserve_paths = [
    "ios/libs/ios-arm64/libcrabytest.a",
    "ios/libs/ios-arm64-simulator/libcrabytest.a"
  ]



  s.pod_target_xcconfig = {
    'HEADER_SEARCH_PATHS' => '$(PODS_TARGET_SRCROOT)/ios/include',
    'OTHER_LDFLAGS[arch=arm64][sdk=iphoneos*]' => '-force_load $(PODS_TARGET_SRCROOT)/ios/libs/ios-arm64/libcrabytest.a -lpthread',
    'OTHER_LDFLAGS[arch=arm64][sdk=iphonesimulator*]' => '-force_load $(PODS_TARGET_SRCROOT)/ios/libs/ios-arm64-simulator/libcrabytest.a -lpthread',
  }

  install_modules_dependencies(s)
end
