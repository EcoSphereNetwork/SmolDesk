module.exports = {
  preset: 'react-native',
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node'],
  transformIgnorePatterns: [
    'node_modules/(?!(react-native|@react-native|react-native-responsive-fontsize|react-native-iphone-x-helper|react-native-gesture-handler|react-native-reanimated)/)'
  ],
};
