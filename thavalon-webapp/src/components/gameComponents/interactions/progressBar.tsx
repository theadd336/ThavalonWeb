import React from "react";
import "../../../styles/gameStyles/interactionStyles/proposalManager.scss";


interface GameProgressBarProps {
    label: string,
    now: number,
}

export function GameProgressBar(props: GameProgressBarProps): JSX.Element {
    return (
        <div className="game-progress">
            <div
                role="progressbar"
                className="game-progress-bar"
                style={{ width: "" + props.now + "%" }}>
            </div>
            <div style={{ position: "absolute" }}>
                {props.label}
            </div>
        </div>
    )
}