import React from 'react';
import ConnectionManager from './ConnectionManager';

export default function ConnectionManagerDemo() {
  return (
    <div style={{ width: 500 }}>
      <ConnectionManager signalingServer="ws://localhost:5173" />
    </div>
  );
}
