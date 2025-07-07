# SmolDesk Mobile

React Native client for the SmolDesk remote desktop service.

## Requirements
- Node.js 18+
- Android Studio or Xcode for platform builds

## Setup
```bash
npm install
```

## Run on Android
```bash
npm run android
```

## Run on iOS
```bash
npm run ios
```

The app uses the existing SmolDesk signaling server to establish a WebRTC connection.

## Configuration
The signaling server URL can be adjusted in `src/config.ts` or by editing the input field on the connect screen at runtime.
