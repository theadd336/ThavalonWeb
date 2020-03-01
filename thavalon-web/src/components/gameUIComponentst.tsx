import * as React from "react";
import { MissionIndicatorCollection } from "./missionIndicators";
import { RoleCaption } from "./roleInformation";
import { WebSocketManager } from "./communication";

interface GameBoardProps {
    missionIndicators: any
}

/**
 * Class for the center game board. Contains mission indicators and role blurb.
 * This high level class only passes websockets to children and maintains no state.
 */
export class GameBoard extends React.Component
{
    constructor(props: {webSocket: WebSocket}) {
        super(props);
    }
    
    render(): JSX.Element {
        return (
            <div className="container pt-5">
                <RoleCaption webSocket={WebSocketManager} />
                <MissionIndicatorCollection />
            </div>
        );
    }
}