import React from 'react';
import { View, StyleSheet, Button } from 'react-native';
import { RTCView, MediaStream } from 'react-native-webrtc';

interface Props {
  stream: MediaStream;
  onDisconnect: () => void;
}

export default function ViewerScreen({ stream, onDisconnect }: Props) {
  return (
    <View style={styles.container}>
      <RTCView
        streamURL={stream.toURL()}
        style={styles.video}
        objectFit="cover"
      />
      <View style={styles.toolbar}>
        <Button title="Verbindung trennen" onPress={onDisconnect} />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#000' },
  video: { flex: 1 },
  toolbar: {
    position: 'absolute',
    bottom: 20,
    left: 0,
    right: 0,
    alignItems: 'center',
  },
});
