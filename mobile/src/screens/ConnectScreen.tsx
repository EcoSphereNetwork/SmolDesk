import React, { useState } from 'react';
import { View, TextInput, Button, StyleSheet } from 'react-native';
import { DEFAULT_SIGNALING_SERVER } from '../config';

interface Props {
  onConnect: (server: string, room: string) => void;
}

export default function ConnectScreen({ onConnect }: Props) {
  const [serverUrl, setServerUrl] = useState(DEFAULT_SIGNALING_SERVER);
  const [roomId, setRoomId] = useState('');

  return (
    <View style={styles.container}>
      <TextInput
        style={styles.input}
        placeholder="Server URL"
        value={serverUrl}
        onChangeText={setServerUrl}
      />
      <TextInput
        style={styles.input}
        placeholder="Raumcode"
        value={roomId}
        onChangeText={setRoomId}
      />
      <Button title="Verbinden" onPress={() => onConnect(serverUrl, roomId)} />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, justifyContent: 'center', padding: 20 },
  input: {
    borderWidth: 1,
    borderColor: '#ccc',
    marginBottom: 10,
    padding: 8,
  },
});
