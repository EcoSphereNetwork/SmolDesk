import React from 'react';
import { View, Button } from 'react-native';
import { authorize } from 'react-native-app-auth';
import * as Keychain from 'react-native-keychain';
import { OAUTH_CONFIG } from '../config';

interface Props {
  onLoggedIn: (token: string) => void;
}

export default function LoginScreen({ onLoggedIn }: Props) {
  const handleLogin = async () => {
    try {
      const result = await authorize(OAUTH_CONFIG);
      const token = result.accessToken;
      if (token) {
        await Keychain.setGenericPassword('oauth', JSON.stringify(result));
        onLoggedIn(token);
      }
    } catch (e) {
      console.warn('Login failed', e);
    }
  };

  return (
    <View style={{ flex: 1, justifyContent: 'center', alignItems: 'center' }}>
      <Button title="Login" onPress={handleLogin} />
    </View>
  );
}
