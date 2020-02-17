function populateGameState(gamestate) {
    if (gamestate === null || gamestate === undefined) { return; }
    // Always populate role information and any previous mission on a reconnect.
    populateRoleBlurb(gamestate.roleInformation);
    populateRoleInformation(gamestate.roleInformation.information, gamestate.roleInformation.role);
    const allMissionInfo = gamestate.missionInfo;
    let count = 0;
    console.log(allMissionInfo);
    if (typeof allMissionInfo === "object") {
        for (const missionNum in allMissionInfo) {
            const missionResults = allMissionInfo[missionNum];
            const missionResult = missionResults.missionResult;
            const playersOnMission = missionResults.playersOnMissions;
            const playedCards = missionResults.playedCards;
            console.log(playersOnMission);
            writeMissionResults(Number(missionNum) + 1, missionResult, playersOnMission, playedCards);
        }
    }
    // What happens next depends on the current game phase.
    switch (gamestate.currentPhase) {
        case 0:
            populateProposalTab(
                gamestate.isProposing,
                gamestate.proposerIndex,
                gamestate.proposalOrder,
                gamestate.proposalNum,
                gamestate.maxNumProposals,
                gamestate.proposalSize,
                gamestate.currentProposal);
            break;
        case 1:
            writeVotingInformation(gamestate.currentProposal);
            break;
    }
}

function onPropose(proposalInfo) {
    writeProposalBodyOther(proposalInfo.proposerName, proposalInfo.proposedPlayerList, proposalInfo.isProposing);
}

function onMoveToVote(message) {
    writeVotingInformation(message.playerList);
}

function newProposal(message) {
    populateProposalTab(
        message.isProposing,
        message.proposerIndex,
        message.proposalOrder,
        message.proposalNum,
        message.maxNumProposals,
        message.proposalSize,
        message.currentProposal);
    if (message.priorVoteInfo != null) {
        writePriorProposalVoteResults(message.priorVoteInfo);
    }
}

function onMissionStart(message) {
    writeMissionStartInfo(message.isOnMission, message.playerList, message.priorVoteInfo);
}

function onMissionResults(message) {
    writeMissionResults(message.priorMissionNum, message.missionResult, message.playersOnMission, message.playedCards);
}

function onVoteStillInProgress(message) {
    writeVoteStillInProgress(message.submittedVote);
}
