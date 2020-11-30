export class GameSocket {
    private static instance: WebSocket;

    /**
     * Get the account manager instance, creating one if needed.
     * 
     * @param socketUrl The url to connect to for the websocket
     * @returns The instance of the AccountManager.
     */
    public static getInstance(socketUrl: string): WebSocket {
        console.log(socketUrl);
        // if (!GameSocket.instance) {
            console.log("making new instance.");
            GameSocket.instance = new WebSocket(socketUrl);
            GameSocket.instance.onmessage = (msg: any) => {
                console.log(msg);
                console.log(msg.data);
            }

            GameSocket.instance.onopen = () => {
                GameSocket.instance.send(JSON.stringify({
                    "message_type": "Ping"
                }));
            };
        // }
        return GameSocket.instance;
    }
}