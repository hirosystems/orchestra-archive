import { SocketContext } from "../components/NetworkingProvider";
import { useContext } from "react";

const useSocket = () => {
  const socket = useContext(SocketContext);

  return socket;
};

export { useSocket };