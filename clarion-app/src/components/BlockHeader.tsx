import { useCallback, useEffect } from "react";
import { useSocket } from "../hooks/useSocket";
import {Heading, Box} from '@primer/react'

const BlockHeader = () => {
  const socket = useSocket();

  const onMessage = useCallback((message) => {
    const data = JSON.parse(message?.data);
    console.log(data);
  }, []);

  useEffect(() => {
    socket.addEventListener("message", onMessage);

    return () => {
      socket.removeEventListener("message", onMessage);
    };
  }, [socket, onMessage]);

  return (
    <Box style={{ width: '100%', height: 200, backgroundColor: '#24292f' }}>
    <Box style={{ width: 300, height: 200, backgroundColor: 'rgba(0,0,0,0.5)' }}>
      <Heading sx={{ pt: 48, pl: 24, color: 'white'}}>Block 1</Heading>
    </Box>
  </Box>
  );
};

export { BlockHeader };