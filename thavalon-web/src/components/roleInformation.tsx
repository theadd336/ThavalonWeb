import * as React from "react";
import { Team } from "../Core/gameConstants";
import { MissingPropertyError } from "../Core/errors";

interface RoleCaptionProps {
    role: string
    team: Team
}

export class RoleCaption extends React.Component<RoleCaptionProps>
{
    constructor(props: RoleCaptionProps) {
        super(props);
        if (typeof props.role !== "string"
            || !(props.team in Team)) {
            throw new MissingPropertyError("Role caption must contain team indicator and role.");
        }
    }

    render(): JSX.Element {
        let teamIndicator: JSX.Element;
        if (this.props.team === Team.Evil) {
            teamIndicator = <span className="text-danger"> [EVIL]</span>;
        } else {
            teamIndicator = <span className="text-success"> [GOOD]</span>;
        }
        return (
            <div className="row col-12 text-center">
                <span>
                    {"You are " + this.props.role} 
                    {teamIndicator}
                </span>
            </div>
        );
    }
}