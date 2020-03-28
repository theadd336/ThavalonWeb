import React from "react";
import { TabComponent } from "./tabComponents";
import { Card, GamePhase } from "../Core/gameConstants";
import { WebSocketProp } from "./communication";
import FailToken from "../static/red-coin.png";
import SuccessToken from "../static/black-coin.png";
import ReverseToken from "../static/circle-png.png";
import { Figure, Button, Container, Row } from "react-bootstrap";
import { OutgoingMessageTypes, IncomingMessage, IncomingMessageTypes, GamePhaseChangeMessage } from "../Core/commConstants";

interface MissionTabState {
    gamePhase: GamePhase
    playersOnMission: string[]
    isOnMission: boolean
    playedCard?: Card
}

interface SubmitMissionCardMessage {
    type: OutgoingMessageTypes.PlayCard;
    playedCard: Card;
}

interface MissionInfoMessage {
    playersOnMission: string[];
    isOnMission: boolean;
    playedCard?: Card;
}

export class MissionTab extends TabComponent<MissionTabState> {
    constructor(props: WebSocketProp) {
        super(props);
        this.state = {
            gamePhase: GamePhase.Proposal,
            playersOnMission: [],
            isOnMission: false
        }
    }

    render(): JSX.Element {
        const { gamePhase, isOnMission, playersOnMission, playedCard } = this.state;
        let tab: JSX.Element;
        if (gamePhase === GamePhase.Mission) {
            tab = this.renderTabForActiveMission(isOnMission, playersOnMission, playedCard);
        } else {
            tab = this.renderWaitingForMission();
        }
        return tab;
    }

    private renderTabForActiveMission(isOnMission: boolean, playersOnMission: string[], playedCard?: Card): JSX.Element {
        let tabContent: JSX.Element;
        if (typeof playedCard !== "undefined") {
            tabContent = this.renderTabAfterCardPlayed(playedCard);
        } else if (isOnMission) {
            tabContent = this.createMissionCards(playersOnMission);
        } else {
            tabContent = this.createTabForObservingPlayer(playersOnMission);
        }
        return tabContent;
    }

    private renderWaitingForMission(): JSX.Element {
        return (
            <span>Please wait for the next mission.</span>
        );
    }

    private createMissionCards(playersOnMission: string[]): JSX.Element {
        let playersOnMissionSentence = "You are on a mission with ";
        playersOnMissionSentence = this.formatPlayerSentence(playersOnMission, playersOnMissionSentence);
        const cards: JSX.Element[] = [];
        for (let i = 0; i < 3; i++) {
            cards.push(this.createFigure(i));
        }

        const cardButtons = cards.map((figure, index) => {
            return (
                <Button
                    type="button"
                    variant="outline-light"
                    onClick={() => { this.playCard(index) }}>
                    {figure}
                </Button>
            );
        });
        return (
            <Container>
                <Row>
                    {playersOnMissionSentence}
                </Row>
                <Row>
                    {cardButtons}
                </Row>
            </Container>
        );
    }

    /**
     * Creates the mission tab for an observing player. This is just a static sentence.
     * @param playersOnMission List of players on the mission.
     */
    private createTabForObservingPlayer(playersOnMission: string[]): JSX.Element {
        let playersOnMissionSentence = "Please wait while ";
        playersOnMissionSentence = this.formatPlayerSentence(playersOnMission, playersOnMissionSentence);
        playersOnMissionSentence += " go on a mission.";
        return <span>{playersOnMissionSentence}</span>
    }

    private formatPlayerSentence(playersOnMission: string[], baseSentence?: string): string {
        if (typeof baseSentence === "undefined") {
            baseSentence = "";
        }
        const numPlayers = playersOnMission.length;
        for (let i = 0; i < numPlayers; i++) {
            const player = playersOnMission[i];
            if (i === numPlayers - 1) {
                baseSentence += "and " + player + ".";
            }
            else if (numPlayers === 2) {
                baseSentence += player + " ";
            }
            else {
                baseSentence += player + ", ";
            }
        }
        return baseSentence;
    }

    private createFigure(card: Card): JSX.Element {
        let image = "";
        switch (card) {
            case Card.Fail:
                image = FailToken;
                break;
            case Card.Reverse:
                image = ReverseToken;
                break;
            case Card.Success:
                image = SuccessToken;
                break;
        }
        const cardImage = (
            <Figure>
                <Figure.Image
                    src={image}
                    height="300"
                    width="300" />
                <Figure.Caption>
                    {Card[card]}
                </Figure.Caption>
            </Figure>
        );
        return cardImage;
    }
    private renderTabAfterCardPlayed(cardPlayed: Card): JSX.Element {
        return <span>You have played a {Card[cardPlayed]}. Please wait for others to finish playing cards.</span>;
    }

    private playCard(card: Card): void {
        const message: SubmitMissionCardMessage = {
            type: OutgoingMessageTypes.PlayCard,
            playedCard: card
        }
        this.sendMessage(message);
        this.setState({ playedCard: card });
    }

    protected receiveSuccessfulMessage(_: object, message: IncomingMessage): void {
        if (message.type === IncomingMessageTypes.MissionInformation) {
            this.updateMissionInfo(message.data as MissionInfoMessage);
        } else if (message.type === IncomingMessageTypes.GamePhaseChange) {
            this.updateGamePhase(message.data as GamePhaseChangeMessage)
        }
    }

    private updateGamePhase(gamePhaseMessage: GamePhaseChangeMessage): void {
        const newState = {
            gamePhase: gamePhaseMessage.gamePhase,
            playedCard: undefined
        }
        this.setState(newState);
    }

    private updateMissionInfo(missionInfo: MissionInfoMessage): void {
        const newState = {
            gamePhase: GamePhase.Mission,
            playersOnMission: missionInfo.playersOnMission,
            isOnMission: missionInfo.isOnMission,
            playedCard: missionInfo.playedCard
        }
        this.setState(newState);
    }
}