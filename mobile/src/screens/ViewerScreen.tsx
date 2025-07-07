import React, { useState, useRef, useEffect } from 'react';
import { SafeAreaView } from 'react-native-safe-area-context';
import { View, StyleSheet, Button, useWindowDimensions } from 'react-native';
import { RFValue } from 'react-native-responsive-fontsize';
import { useTheme } from 'react-native-paper';
import {
  PanGestureHandler,
  PinchGestureHandler,
  TapGestureHandler,
} from 'react-native-gesture-handler';
import Animated, {
  useSharedValue,
  useAnimatedStyle,
  useAnimatedGestureHandler,
  withTiming,
  runOnJS,
} from 'react-native-reanimated';
import { RTCView, MediaStream } from 'react-native-webrtc';
import WebRTCService from '../services/webrtc';
import SignalingService, { MonitorInfo } from '../services/signaling';
import FileTransferService from '../services/files';
import MonitorSelector from '../components/MonitorSelector';
import TouchToMouse from '../input/touchToMouse';

interface Props {
  stream: MediaStream;
  service: WebRTCService;
  signaling: SignalingService;
  onDisconnect: () => void;
}
export default function ViewerScreen({ stream, service, signaling, onDisconnect }: Props) {
  const { width, height } = useWindowDimensions();
  const mouse = new TouchToMouse(service);
  const { colors } = useTheme();

  const scale = useSharedValue(1);
  const translateX = useSharedValue(0);
  const translateY = useSharedValue(0);
  const [pointerMode, setPointerMode] = useState(false);
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [selectorVisible, setSelectorVisible] = useState(false);
  const fileService = useRef<FileTransferService>();

  useEffect(() => {
    fileService.current = new FileTransferService(service);
    const handleMonitors = (list: MonitorInfo[]) => setMonitors(list);
    signaling.on('monitors', handleMonitors);
    return () => {
      signaling.off('monitors', handleMonitors);
    };
  }, [service, signaling]);

  const pinchHandler = useAnimatedGestureHandler({
    onStart: (_, ctx: any) => {
      ctx.start = scale.value;
    },
    onActive: (e, ctx: any) => {
      scale.value = ctx.start * e.scale;
    },
  });

  const panHandler = useAnimatedGestureHandler({
    onStart: (_, ctx: any) => {
      ctx.x = translateX.value;
      ctx.y = translateY.value;
    },
    onActive: (e, ctx: any) => {
      if (pointerMode && scale.value <= 1) {
        const dist = Math.sqrt(e.translationX * e.translationX + e.translationY * e.translationY);
        if (dist > 5) runOnJS(mouse.move)(e.translationX, e.translationY);
        return;
      }
      translateX.value = ctx.x + e.translationX;
      translateY.value = ctx.y + e.translationY;
    },
  });

  const doubleTap = useAnimatedGestureHandler({
    onActive: () => {
      const next = scale.value > 1 ? 1 : 2;
      scale.value = withTiming(next);
      translateX.value = withTiming(0);
      translateY.value = withTiming(0);
    },
  });

  const style = useAnimatedStyle(() => ({
    transform: [
      { scale: scale.value },
      { translateX: translateX.value },
      { translateY: translateY.value },
    ],
    width,
    height,
  }));

  return (
    <View style={[styles.container, { backgroundColor: colors.background }] }>
      <TapGestureHandler onGestureEvent={doubleTap} numberOfTaps={2}>
        <PinchGestureHandler onGestureEvent={pinchHandler}>
          <PanGestureHandler onGestureEvent={panHandler} minPointers={1}>
            <Animated.View style={style}>
              <RTCView
                streamURL={stream.toURL()}
                style={{ width: '100%', height: '100%' }}
                objectFit="cover"
              />
            </Animated.View>
          </PanGestureHandler>
        </PinchGestureHandler>
      </TapGestureHandler>
      <SafeAreaView edges={['bottom']} style={styles.toolbar}>
        <Button
          title={pointerMode ? 'Zeiger' : 'Scroll'}
          color={pointerMode ? colors.primary : undefined}
          onPress={() => setPointerMode(!pointerMode)}
        />
        <Button title="Datei senden" onPress={() => fileService.current?.pickAndSend()} />
        <Button title="Monitor wechseln" onPress={() => setSelectorVisible(true)} />
        <Button title="Verbindung trennen" onPress={onDisconnect} />
      </SafeAreaView>
      <MonitorSelector
        visible={selectorVisible}
        monitors={monitors}
        onSelect={(id) => {
          signaling.selectMonitor(id);
          setSelectorVisible(false);
        }}
        onClose={() => setSelectorVisible(false)}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  video: { flex: 1 },
  toolbar: {
    position: 'absolute',
    bottom: RFValue(20),
    left: 0,
    right: 0,
    alignItems: 'center',
    gap: 8,
  },
});
