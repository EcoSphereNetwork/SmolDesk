import React from 'react';
import RemoteScreen from './RemoteScreen';

export default function RemoteScreenDemo() {
  return (
    <div style={{ width: 800, height: 600 }}>
      <RemoteScreen isConnected={false} />
    </div>
  );
}
