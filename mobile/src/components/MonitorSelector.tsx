import React from 'react';
import { Modal, View, Button, StyleSheet } from 'react-native';
import Toast from 'react-native-toast-message';
import { MonitorInfo } from '../services/signaling';

interface Props {
  visible: boolean;
  monitors: MonitorInfo[];
  onSelect: (id: number) => void;
  onClose: () => void;
}

export default function MonitorSelector({ visible, monitors, onSelect, onClose }: Props) {
  return (
    <Modal visible={visible} transparent animationType="fade">
      <View style={styles.overlay}>
        {monitors.map((m) => (
          <Button
            key={m.id}
            title={m.name || `${m.width}x${m.height}`}
            onPress={() => {
              Toast.show({ type: 'info', text1: `Monitor ${m.id}` });
              onSelect(m.id);
            }}
          />
        ))}
        <Button title="Abbrechen" onPress={onClose} />
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: 'rgba(0,0,0,0.5)',
  },
});
