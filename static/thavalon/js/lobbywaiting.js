function onPlayerJoinLeave(data) {
	// Get the player list array from data. If it's undefined, just return.
	const playerNames = data["player_names"];
	if (playerNames === undefined) {
		return;
	}
	// Set up variables. For each player in the player list, create the HTML string to set as the inner HTML. 
	let playerHTMLString = "";
	const templateList = document.getElementById("waitingPlayers");
	for (let player of playerNames) {
		console.log(player)
		playerHTMLString = createHTMLString(playerHTMLString, player);
	}
	// Finally, add the lobby warning and set the list HTML to the player list.
	playerHTMLString += "<li class=\"list-group-item text-warning\"><span>Waiting for Lobby Leader to start game</span></li>";
	templateList.innerHTML = playerHTMLString;
	return;
}


function createHTMLString(playerHTMLString, playerName) {
	// Append a new list line of HTML to the playerHTMLString and return the string.
	return playerHTMLString + "<li class=\"list-group-item\">" + playerName + "</li>";
}


