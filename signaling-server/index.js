// signaling-server/index.js
const WebSocket = require('ws');
const http = require('http');
const crypto = require('crypto');
const { v4: uuidv4 } = require('uuid');

// Configuration
const PORT = process.env.PORT || 3000;
const HEARTBEAT_INTERVAL = 30000; // 30 seconds
const CONNECTION_TIMEOUT = 120000; // 2 minutes

// Create HTTP server
const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('SmolDesk Signaling Server');
});

// Create WebSocket server
const wss = new WebSocket.Server({ server });

// Room management
const rooms = new Map();
const clients = new Map();

// Generate session token
function generateToken() {
  return crypto.randomBytes(32).toString('hex');
}

// Log with timestamp
function log(message) {
  console.log(`[${new Date().toISOString()}] ${message}`);
}

// Handle new WebSocket connection
wss.on('connection', (ws, req) => {
  const clientId = uuidv4();
  const token = generateToken();
  
  // Store client information
  clients.set(ws, {
    id: clientId,
    token: token,
    roomId: null,
    isAlive: true,
    lastActivity: Date.now()
  });
  
  log(`New connection: ${clientId}`);
  
  // Send client their ID and token
  ws.send(JSON.stringify({
    type: 'welcome',
    clientId: clientId,
    token: token
  }));
  
  // Set up ping/pong for connection monitoring
  ws.on('pong', () => {
    const client = clients.get(ws);
    if (client) {
      client.isAlive = true;
      client.lastActivity = Date.now();
    }
  });
  
  // Handle messages
  ws.on('message', (message) => {
    try {
      const data = JSON.parse(message);
      const client = clients.get(ws);
      
      if (!client) {
        return;
      }
      
      // Update last activity timestamp
      client.lastActivity = Date.now();
      
      // Handle different message types
      switch (data.type) {
        case 'create-room':
          handleCreateRoom(ws, client, data);
          break;
        
        case 'join-room':
          handleJoinRoom(ws, client, data);
          break;
        
        case 'leave-room':
          handleLeaveRoom(ws, client);
          break;
        
        case 'ice-candidate':
          handleIceCandidate(client, data);
          break;
        
        case 'offer':
          handleOffer(client, data);
          break;
        
        case 'answer':
          handleAnswer(client, data);
          break;
        
        case 'ping':
          // Simple ping/pong to keep connection alive
          ws.send(JSON.stringify({ type: 'pong' }));
          break;
        
        default:
          log(`Unknown message type: ${data.type}`);
      }
    } catch (error) {
      log(`Error processing message: ${error.message}`);
    }
  });
  
  // Handle disconnection
  ws.on('close', () => {
    const client = clients.get(ws);
    
    if (client) {
      log(`Connection closed: ${client.id}`);
      
      // Remove client from room if they're in one
      if (client.roomId && rooms.has(client.roomId)) {
        const room = rooms.get(client.roomId);
        
        // Notify other clients in the room
        for (const [peerWs, peerInfo] of clients.entries()) {
          if (peerInfo.roomId === client.roomId && peerWs !== ws) {
            peerWs.send(JSON.stringify({
              type: 'peer-disconnected',
              peerId: client.id
            }));
          }
        }
        
        // Remove client from room
        room.clients = room.clients.filter(id => id !== client.id);
        
        // Delete room if empty
        if (room.clients.length === 0) {
          rooms.delete(client.roomId);
          log(`Room deleted: ${client.roomId}`);
        }
      }
      
      // Remove client from clients map
      clients.delete(ws);
    }
  });
});

// Room management handlers
function handleCreateRoom(ws, client, data) {
  const roomId = data.roomId || uuidv4();
  
  // Check if room already exists
  if (rooms.has(roomId)) {
    ws.send(JSON.stringify({
      type: 'error',
      message: 'Room already exists'
    }));
    return;
  }
  
  // Create new room
  rooms.set(roomId, {
    id: roomId,
    host: client.id,
    clients: [client.id],
    created: Date.now(),
    settings: data.settings || {}
  });
  
  // Update client with room info
  client.roomId = roomId;
  
  // Send confirmation
  ws.send(JSON.stringify({
    type: 'room-created',
    roomId: roomId
  }));
  
  log(`Room created: ${roomId} by client ${client.id}`);
}

function handleJoinRoom(ws, client, data) {
  const { roomId } = data;
  
  // Check if room exists
  if (!rooms.has(roomId)) {
    ws.send(JSON.stringify({
      type: 'error',
      message: 'Room not found'
    }));
    return;
  }
  
  const room = rooms.get(roomId);
  
  // Add client to room
  room.clients.push(client.id);
  client.roomId = roomId;
  
  // Notify client they've joined
  ws.send(JSON.stringify({
    type: 'room-joined',
    roomId: roomId,
    peers: room.clients.filter(id => id !== client.id),
    settings: room.settings
  }));
  
  // Notify other clients in the room
  for (const [peerWs, peerInfo] of clients.entries()) {
    if (peerInfo.roomId === roomId && peerWs !== ws) {
      peerWs.send(JSON.stringify({
        type: 'peer-joined',
        peerId: client.id
      }));
    }
  }
  
  log(`Client ${client.id} joined room ${roomId}`);
}

function handleLeaveRoom(ws, client) {
  if (!client.roomId || !rooms.has(client.roomId)) {
    return;
  }
  
  const roomId = client.roomId;
  const room = rooms.get(roomId);
  
  // Remove client from room
  room.clients = room.clients.filter(id => id !== client.id);
  
  // Notify other clients
  for (const [peerWs, peerInfo] of clients.entries()) {
    if (peerInfo.roomId === roomId && peerWs !== ws) {
      peerWs.send(JSON.stringify({
        type: 'peer-left',
        peerId: client.id
      }));
    }
  }
  
  // Update client
  client.roomId = null;
  
  // Send confirmation
  ws.send(JSON.stringify({
    type: 'room-left',
    roomId: roomId
  }));
  
  // Delete room if empty
  if (room.clients.length === 0) {
    rooms.delete(roomId);
    log(`Room deleted: ${roomId}`);
  }
  
  log(`Client ${client.id} left room ${roomId}`);
}

// WebRTC signaling handlers
function handleIceCandidate(client, data) {
  const { candidate, targetId } = data;
  
  if (!client.roomId || !targetId) {
    return;
  }
  
  forwardToClient(targetId, {
    type: 'ice-candidate',
    candidate: candidate,
    peerId: client.id
  });
}

function handleOffer(client, data) {
  const { offer, targetId } = data;
  
  if (!client.roomId || !targetId) {
    return;
  }
  
  forwardToClient(targetId, {
    type: 'offer',
    offer: offer,
    peerId: client.id
  });
}

function handleAnswer(client, data) {
  const { answer, targetId } = data;
  
  if (!client.roomId || !targetId) {
    return;
  }
  
  forwardToClient(targetId, {
    type: 'answer',
    answer: answer,
    peerId: client.id
  });
}

// Helper function to forward messages to a specific client
function forwardToClient(targetId, message) {
  for (const [ws, client] of clients.entries()) {
    if (client.id === targetId) {
      ws.send(JSON.stringify(message));
      return true;
    }
  }
  return false;
}

// Set up heartbeat to detect dead connections
const heartbeat = setInterval(() => {
  const now = Date.now();
  
  for (const [ws, client] of clients.entries()) {
    // Check if client has timed out
    if (now - client.lastActivity > CONNECTION_TIMEOUT) {
      log(`Client ${client.id} timed out`);
      ws.terminate();
      continue;
    }
    
    // Send ping to check if client is still alive
    if (client.isAlive === false) {
      // No response to previous ping, terminate
      log(`Client ${client.id} not responding`);
      ws.terminate();
      continue;
    }
    
    // Mark as not alive until pong is received
    client.isAlive = false;
    ws.ping();
  }
  
  // Clean up empty rooms
  for (const [roomId, room] of rooms.entries()) {
    if (room.clients.length === 0) {
      rooms.delete(roomId);
      log(`Room deleted: ${roomId}`);
    }
  }
}, HEARTBEAT_INTERVAL);

// Clean up on server close
server.on('close', () => {
  clearInterval(heartbeat);
});

// Start server
server.listen(PORT, () => {
  log(`Signaling server running on port ${PORT}`);
});

// Handle termination signals
process.on('SIGINT', () => {
  log('Server shutting down');
  wss.close();
  server.close();
  process.exit(0);
});

// Export for testing
module.exports = { server, wss };
