import React from "react";
import { SelectedPlayerType, Vote } from "./constants";
import { Spinner } from "react-bootstrap";

/**
 * Props object for the PlayerCard component. Some of these aren't used yet,
 * pending future development.
 */
interface PlayerCardProps {
    name: string
    className?: string
    toggleSelected?: (name: string) => void,
    me?: boolean,
    tabbedOut?: boolean,
    isProposing?: boolean,
    vote?: Vote,
    declaredAs?: string,
    enabled?: boolean,
    selectedTypes?: SelectedPlayerType[]
}

/**
 * React component representing an interactive player button. This button doesn't
 * directly communicate with the server but handles all styling of relevant icons.
 * @param props The props for the player card
 */
export function PlayerCard(props: PlayerCardProps): JSX.Element {
    let disabled = false;
    if (props.enabled === false) {
        disabled = true;
    }

    let selectedClassNames = new Array<string>();
    if (props.selectedTypes !== undefined) {
        for (const selectedType of props.selectedTypes) {
            switch (selectedType) {
                case SelectedPlayerType.Primary:
                    selectedClassNames.push("player-selected-primary");
                    break;
                case SelectedPlayerType.Secondary:
                    selectedClassNames.push("player-selected-secondary");
                    break;
            }
        }
        if (selectedClassNames.length > 1) {
            selectedClassNames = ["player-selected-all"];
        }
    }

    let voteLetter: string = "";
    if (props.vote === Vote.Upvote) {
        voteLetter = "Upvoted";
    } else if (props.vote === Vote.Downvote) {
        voteLetter = "Downvoted";
    }

    const className = props.className === undefined ? "" : props.className;
    const baseClassName = className !== "" ? className : "player-card-base";
    return (
        <button
            disabled={disabled}
            className={`${ baseClassName } ${ selectedClassNames.join(" ") }`}
            onClick={() => {
                if (props.toggleSelected !== undefined) {
                    props.toggleSelected(props.name);
                }
            }}>
            <div>
                <div style={{
                    position: "relative", float: "left"
                }}>
                    {voteLetter}
                </div>
                {props.tabbedOut &&
                    <Spinner
                        className="tabbed-out-indicator"
                        size="sm"
                        variant="dark"
                        animation="border" />}
                {props.name}
                {props.declaredAs !== undefined ? ` - ${ props.declaredAs }` : ""}
                <div style={{ position: "relative", float: "right" }}>
                    {props.isProposing ? "Proposing" : ""}
                </div>
            </div>
        </button>
    );
}