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

  # TODO: Auto configure xcframework path
  s.vendored_frameworks = "ios/framework/libcrabytest.xcframework"

  install_modules_dependencies(s)
end
