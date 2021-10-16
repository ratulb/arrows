import java.io.*;
import java.net.*;

public class QuoteClient {

	
public static void main(String[] args)  throws Exception {
  //System.out.println("Hello");
  //byte[] buf = new byte[256];
  byte[] buf = "Hello".getBytes();
  DatagramSocket socket = new DatagramSocket();
  InetAddress address = InetAddress.getLocalHost();
 // InetAddress address = InetAddress.anyLocalAddress();
  DatagramPacket packet = new DatagramPacket(buf, buf.length, address, 7171);
  socket.send(packet);
  packet = new DatagramPacket(buf, buf.length);
  socket.receive(packet);
  String received = new String(packet.getData(), 0, packet.getLength());
  System.out.println("Received: " + received);
}
}
