import { AppRegistry } from 'react-native';
import { App } from 'test-suites';
import { name as appName } from './app.json';

AppRegistry.registerComponent(appName, () => App);
