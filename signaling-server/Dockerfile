# signaling-server/Dockerfile
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --production

COPY . .

EXPOSE 3000
USER node

CMD ["node", "index.js"]
