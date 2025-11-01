Pod::Spec.new do |s|
  s.name         = "CrabyTest"
  s.version      = "0.1.0"
  s.summary      = "Craby test module"
  s.homepage     = "https://github.com/leegeunhyeok/craby"
  s.license      = "MIT"
  s.authors      = "leegeunhyeok <dev.ghlee@gmail.com> (https://github.com/leegeunhyeok)"

  s.platforms    = { :ios => min_ios_version_supported }
  s.source       = { :git => "https://github.com/leegeunhyeok/craby.git", :tag => "#{s.version}" }

  s.source_files = ["ios/**/*.{m,mm,cc,cpp}", "cpp/**/*.cpp"]
  s.vendored_frameworks = "ios/framework/libcrabytest.xcframework"
  s.pod_target_xcconfig = {
    "HEADER_SEARCH_PATHS" => [
      '"${PODS_TARGET_SRCROOT}/cpp"',
      '"${PODS_TARGET_SRCROOT}/ios/include"',
    ].join(' '),
    "CLANG_CXX_LANGUAGE_STANDARD" => "c++20",
  }

  install_modules_dependencies(s)
end
