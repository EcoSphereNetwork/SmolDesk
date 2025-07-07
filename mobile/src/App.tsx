import React, { useState, useEffect } from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import ConnectScreen from './screens/ConnectScreen';
import ViewerScreen from './screens/ViewerScreen';
import LoginScreen from './screens/LoginScreen';
import SignalingService from './services/signaling';
import WebRTCService from './services/webrtc';
import * as Keychain from 'react-native-keychain';
import CryptoJS from 'crypto-js';
import { ENCRYPTION_KEY_SALT, HMAC_ENABLED, HMAC_KEY } from './config';
import { MediaStream } from 'react-native-webrtc';

const Stack = createNativeStackNavigator();

export default function App() {
  const [webrtc, setWebrtc] = useState<WebRTCService | null>(null);
  const [signaling, setSignaling] = useState<SignalingService | null>(null);
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [token, setToken] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      const creds = await Keychain.getGenericPassword();
      if (creds) {
        try {
          const obj = JSON.parse(creds.password);
          setToken(obj.accessToken);
        } catch {
          await Keychain.resetGenericPassword();
        }
      }
    })();
  }, []);

  const handleConnect = (server: string, room: string) => {
    const key = CryptoJS.PBKDF2(token || '', ENCRYPTION_KEY_SALT, { keySize: 32/4 }).toString();
    const sig = new SignalingService({ url: server, token: token || undefined, hmacKey: HMAC_ENABLED ? HMAC_KEY : undefined });
    const service = new WebRTCService({ signaling: sig, encryptionKey: key });
    service.on('stream', setStream);
    sig.on('close', () => {
      setStream(null);
      setWebrtc(null);
      setSignaling(null);
    });
    sig.on('unauthorized', () => {
      setToken(null);
    });
    service.join(room);
    setWebrtc(service);
    setSignaling(sig);
  };

  const handleDisconnect = () => {
    webrtc?.disconnect();
    signaling?.disconnect();
    setStream(null);
    setWebrtc(null);
    setSignaling(null);
  };

  return (
    <NavigationContainer>
      <Stack.Navigator screenOptions={{ headerShown: false }}>
        {stream ? (
          <Stack.Screen name="viewer">
            {() => (
              <ViewerScreen
                stream={stream!}
                service={webrtc!}
                signaling={signaling!}
                onDisconnect={handleDisconnect}
              />
            )}
          </Stack.Screen>
        ) : token ? (
          <Stack.Screen name="connect">
            {() => <ConnectScreen onConnect={handleConnect} />}
          </Stack.Screen>
        ) : (
          <Stack.Screen name="login">
            {() => <LoginScreen onLoggedIn={setToken} />}
          </Stack.Screen>
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
}
