import React, { useEffect } from "react";
import { Container, Col, Row } from "react-bootstrap";

export function GameRoot(): JSX.Element {
    useEffect(() => {
        // componentDidMount -> Get all game snapshots
        // componentWillUnmount -> Nothing.
    }, []);

    return (
        <Container showGrid
            className="game-root-container">
            <Row>
                <Col xs={4}>
                    <h1>Left Column</h1>
                    <Row
                        className="border-bottom"
                        xs={5}>
                        <h1>Top Row</h1>
                    </Row>
                    <Row>
                        <h1>Bottom Row</h1>
                    </Row>
                </Col>
                <Col className="border-left">
                    <h1>Right Column</h1>
                </Col>
            </Row>
        </Container>
    );
}