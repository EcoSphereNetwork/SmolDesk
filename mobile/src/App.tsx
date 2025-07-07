import React, { useState } from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import ConnectScreen from './screens/ConnectScreen';
import ViewerScreen from './screens/ViewerScreen';
import SignalingService from './services/signaling';
import WebRTCService from './services/webrtc';
import { MediaStream } from 'react-native-webrtc';

const Stack = createNativeStackNavigator();

export default function App() {
  const [webrtc, setWebrtc] = useState<WebRTCService | null>(null);
  const [stream, setStream] = useState<MediaStream | null>(null);

  const handleConnect = (server: string, room: string) => {
    const signaling = new SignalingService({ url: server });
    const service = new WebRTCService({ signaling });
    service.on('stream', setStream);
    signaling.on('close', () => {
      setStream(null);
      setWebrtc(null);
    });
    service.join(room);
    setWebrtc(service);
  };

  const handleDisconnect = () => {
    webrtc?.disconnect();
    setStream(null);
    setWebrtc(null);
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
                onDisconnect={handleDisconnect}
              />
            )}
          </Stack.Screen>
        ) : (
          <Stack.Screen name="connect">
            {() => <ConnectScreen onConnect={handleConnect} />}
          </Stack.Screen>
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
}
