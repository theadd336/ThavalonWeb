import { Team } from "./gameConstants";

export class GameView {
    private _proposalVoteTab: HTMLElement;
    private _voteForm: HTMLElement;
    private _proposalVoteHeaderSection: HTMLElement;
    private _proposalVoteContent: HTMLElement;
    private _proposalListLocation: HTMLElement;
    //#region constructors
    constructor() {
        this._proposalVoteTab = document.getElementById("nav-profile-tab");
        this._proposalVoteHeaderSection = document.getElementById("proposalVoteHeader");
        this._proposalVoteContent = document.getElementById("proposalVoteContent");
        this._proposalListLocation = document.getElementById("proposalListLocation");        

        if (this._proposalVoteTab === undefined ||
            this._proposalVoteHeaderSection === undefined ||
            this._proposalVoteContent === undefined || 
            this._proposalListLocation === undefined) {
            throw new Error("Document is missing key nodes.");
        }
    }
    //#endregion
    //#region public methods
    /**
     * Populates the player order tab given a list of players.
     * @param playerOrder A list of player names in proposal order.
     */
    populatePlayerOrder(playerOrder: string[]): void {
        // If the list is empty, just return.
        if (playerOrder.length === 0) {
            return;
        }
        // Get the player order location node. For each name in the list, 
        // create a new list entry, add the name, and append the list entry 
        // to the list.
        const playerOrderNode = document.getElementById("playerOrderLocation");
        for (let i = 0; i < playerOrder.length; i++) {
            const playerListEntry = this.createHTMLElement("LI", i.toString());
            playerListEntry.textContent = playerOrder[i];
            playerOrderNode.appendChild(playerListEntry);
        }
    }

    populateAllRoleInformation(role: string, team: Team, information: string): void {
        // Populate both the role blurb and the role information tab.
        // This is static and should only be called on a reconnect or game start.
        this.populateRoleBlurb(role, team);
        this.populateRoleInformationTab(information);
    }
    
    //#endregion
    //#region private methods
    /**
     * Populates the role blurb above the mission indicators
     * @param role The name of the role.
     * @param team Team of the role (Good or Evil).
     */
    private populateRoleBlurb(role: string, team: Team): void {
        const roleBlurbTemplate = <HTMLTemplateElement> document.getElementById("roleBlurbTemplate");
        const roleBlurbLocation = document.getElementById("roleBlurbLocation");
        if (typeof roleBlurbTemplate === "undefined" ||
            typeof roleBlurbLocation === "undefined") {
            throw new Error("Could not locate role information locations.");
        }
        // Clone the template and find all spans.
        const roleBlurb = <HTMLElement>roleBlurbTemplate.content.cloneNode(true);
        const spans = roleBlurb.querySelectorAll("span");
        // Add the role information to the blurb.
        spans[0].textContent = "You are " + role;
        // Add the team information.
        let teamString = "";
        if (team === Team.Evil) {
            teamString = " [EVIL]";
            spans[1].classList.add("text-danger");
        }
        else {
            teamString = " [GOOD]";
            spans[1].classList.add("text-success");
        }
        // Finally, add everything to the role location.
        spans[1].textContent = teamString;
        spans[0].appendChild(spans[1]);
        roleBlurbLocation.appendChild(roleBlurb);
    }

    /**
     * Populates the role information tab with the formatted information string.
     * @param information Formatted information for display
     */
    private populateRoleInformationTab(information: string): void {
        const roleInformationTabLocation = document.getElementById("roleInformationLocationTemplate");
        roleInformationTabLocation.textContent = "-------------------------\r\n";
        roleInformationTabLocation.textContent += information + "\r\n";
        roleInformationTabLocation.textContent += "-------------------------";
    }

    /**
     * Creates an HTML element node using the information given.
     * The type of element is required. All else is optional.
     * @param elementType The HTML tag name
     * @param id ID for the tag
     * @param classList List of classes to add
     */
    private createHTMLElement(elementType: string, id?: string, classList?: string[]): HTMLElement {
        if (typeof elementType !== "string" || elementType.length === 0) {
            throw new Error("Cannot create HTML element with no type.");
        }
        // Initialize the node.
        const node = document.createElement(elementType);
        // Handle ID if passed in.
        if (typeof id === "string") {
            node.id = id;
        }

        // Handle any classes to add.
        if (classList instanceof Array && classList.length > 0) {
            node.classList.add(classList.join(","));
        }

        return node;
    }

    private populateProposalHeader(isProposing: boolean, proposerName: string, proposalNumber: number, maxNumProposals: number, numOnMission: number, skipForceIndicator = false): void {
        this._proposalVoteTab.textContent = "Proposals";

        // Write the header section. This includes proposal number, force indicator,
        // and who is proposing.
        this._proposalVoteHeaderSection.innerHTML = "";
        this._proposalVoteHeaderSection.textContent = `Proposal ${proposalNumber}/${maxNumProposals}`;
        if (proposalNumber === maxNumProposals && skipForceIndicator) {
            const forceIndicatorNode = this.createHTMLElement("SPAN", undefined, ["text-danger"]);
            forceIndicatorNode.textContent = "[FORCE]";
            this._proposalVoteHeaderSection.appendChild(forceIndicatorNode);
        }

        // Blank line for formatting
        this._proposalVoteHeaderSection.appendChild(this.createHTMLElement("BR"));

        // Write the sentence on whom is proposing with correct grammar.
        let proposerSentence = "";
        if (isProposing) {
            proposerSentence += "You are ";
        } else {
            proposerSentence += proposerName + " is ";
        }
        proposerSentence += `proposing a ${numOnMission} person mission.`;
        const sentenceTextNode = document.createTextNode(proposerSentence);
        this._proposalVoteHeaderSection.appendChild(sentenceTextNode);
    }

    private writeProposalBodyOther(proposerName: string, currentProposal: string[], isProposing: boolean): void {
        if (!isProposing) {
        }
    }
    //#endregion
}