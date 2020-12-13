import React, { useEffect } from "react";
import { Container, ListGroup, Button } from "react-bootstrap";

/**
 * Component listing players currently in the lobby and a button to start the game.
 */
export function Lobby(props: any): JSX.Element {
    // useEffect handles componentDidMount and componentWillUnmount
    useEffect(() => {
        // Equivalent here to componentDidMount
        // get WS

        return () => {
            // Equivalent here to componentWillUnmount
            // unsubscribe from the handler. 
        }
    }, []);

    const players = ["a", "b"].map((player) =>
        <ListGroup.Item key={player}>{player}</ListGroup.Item>
    );

    return (
        <Container>
            <h1>Friend Code: {props.friendCode}</h1>
            <ListGroup variant="flush">
                {players}
            </ListGroup>
            <Button
                variant="primary"
                onClick={() => { console.log("hi") }}>
                Start Game
            </Button>
        </Container>
    );
}