import React, { useState } from "react";
import { Lobby } from "./gameComponents/lobby";

export function GameContainer(props: any): JSX.Element {
    const [friendCode, setFriendCode] = useState("");
    const [socketUrl, setSocketUrl] = useState("");

    return <Lobby friendCode={friendCode} />;
}