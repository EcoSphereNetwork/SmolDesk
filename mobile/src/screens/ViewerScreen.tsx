import React, { useState } from 'react';
import { View, StyleSheet, Button, useWindowDimensions } from 'react-native';
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
import TouchToMouse from '../input/touchToMouse';

interface Props {
  stream: MediaStream;
  service: WebRTCService;
  onDisconnect: () => void;
}
export default function ViewerScreen({ stream, service, onDisconnect }: Props) {
  const { width, height } = useWindowDimensions();
  const mouse = new TouchToMouse(service);

  const scale = useSharedValue(1);
  const translateX = useSharedValue(0);
  const translateY = useSharedValue(0);
  const [pointerMode, setPointerMode] = useState(false);

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
        runOnJS(mouse.move)(e.translationX, e.translationY);
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
    <View style={styles.container}>
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
      <View style={styles.toolbar}>
        <Button title="Toggle Mode" onPress={() => setPointerMode(!pointerMode)} />
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
