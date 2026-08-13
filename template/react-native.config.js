/**
 * @see docs https://github.com/react-native-community/cli/blob/main/docs/dependencies.md
 * @type {import('@react-native-community/cli-types').UserDependencyConfig & {
 *   spm: unknown
 * }}
 */
const config = {
  dependency: {
    platforms: {
      android: {
        /**
         * Configured by Craby. DO NOT EDIT.
         *
         * Craby builds artifacts using its own CMakeLists, making the CMakeLists generated during AutoLinking unnecessary.
         * The C++ implementation serves as a no-op and is not actually used.
         *
         * However, React Native autolinking does not support linking only the `PackageList` without a CMakeLists file.
         * To address this, I provide stub files that do not interfere with autolinking behavior.
         */
        cmakeListsPath: 'stubs/CMakeLists.txt',
        libraryName: '{{ pascal_name }}_stub',
      },
      ios: {},
    },
  },

  /**
   * React Native 0.87+ reads this metadata only when the consuming app opts into SwiftPM.
   * Declaring it does not enable SwiftPM or affect CocoaPods consumers.
   */
  spm: {
    name: '{{ pascal_name }}',
    scaffold: false,
  },
};

module.exports = config;
