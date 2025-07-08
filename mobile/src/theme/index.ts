import { useColorScheme, ColorSchemeName } from 'react-native';
import { MD3LightTheme, MD3DarkTheme } from 'react-native-paper';
import lightTheme from './lightTheme';
import darkTheme from './darkTheme';

export const getTheme = (scheme: ColorSchemeName | null) =>
  scheme === 'dark'
    ? { ...MD3DarkTheme, ...darkTheme }
    : { ...MD3LightTheme, ...lightTheme };

export const useAppTheme = () => {
  const scheme = useColorScheme();
  return getTheme(scheme);
};

