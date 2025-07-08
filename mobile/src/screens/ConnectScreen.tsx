import React, { useState } from 'react';
import { SafeAreaView } from 'react-native-safe-area-context';
import { View, TextInput, Button, StyleSheet } from 'react-native';
import { useTheme } from 'react-native-paper';
import { RFValue } from 'react-native-responsive-fontsize';
import { DEFAULT_SIGNALING_SERVER } from '../config';

interface Props {
  onConnect: (server: string, room: string) => void;
}

export default function ConnectScreen({ onConnect }: Props) {
  const [serverUrl, setServerUrl] = useState(DEFAULT_SIGNALING_SERVER);
  const [roomId, setRoomId] = useState('');
  const { colors } = useTheme();

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: colors.background }]}>
      <TextInput
        style={[styles.input, { color: colors.text, borderColor: colors.primary }]}
        placeholder="Server URL"
        value={serverUrl}
        onChangeText={setServerUrl}
      />
      <TextInput
        style={[styles.input, { color: colors.text, borderColor: colors.primary }]}
        placeholder="Raumcode"
        value={roomId}
        onChangeText={setRoomId}
      />
      <Button title="Verbinden" onPress={() => onConnect(serverUrl, roomId)} />
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, justifyContent: 'center', padding: RFValue(20) },
  input: {
    borderWidth: 1,
    borderColor: '#ccc',
    marginBottom: 10,
    padding: RFValue(8),
    fontSize: RFValue(14),
  },
});
