var ThavalonWeb;
(function (ThavalonWeb) {
    var Game;
    (function (Game) {
        var Constants;
        (function (Constants) {
            let Card;
            (function (Card) {
                Card[Card["Success"] = 0] = "Success";
                Card[Card["Fail"] = 1] = "Fail";
                Card[Card["Reverse"] = 2] = "Reverse";
            })(Card = Constants.Card || (Constants.Card = {}));
            let GamePhase;
            (function (GamePhase) {
                GamePhase[GamePhase["Proposal"] = 0] = "Proposal";
                GamePhase[GamePhase["Voting"] = 1] = "Voting";
                GamePhase[GamePhase["Mission"] = 2] = "Mission";
                GamePhase[GamePhase["Assassination"] = 3] = "Assassination";
            })(GamePhase = Constants.GamePhase || (Constants.GamePhase = {}));
            let Vote;
            (function (Vote) {
                Vote[Vote["Downvote"] = 0] = "Downvote";
                Vote[Vote["Upvote"] = 1] = "Upvote";
            })(Vote = Constants.Vote || (Constants.Vote = {}));
            let Team;
            (function (Team) {
                Team[Team["Good"] = 0] = "Good";
                Team[Team["Evil"] = 1] = "Evil";
            })(Team = Constants.Team || (Constants.Team = {}));
            let MissionResult;
            (function (MissionResult) {
                MissionResult[MissionResult["Pass"] = 0] = "Pass";
                MissionResult[MissionResult["Fail"] = 1] = "Fail";
            })(MissionResult = Constants.MissionResult || (Constants.MissionResult = {}));
        })(Constants = Game.Constants || (Game.Constants = {}));
    })(Game = ThavalonWeb.Game || (ThavalonWeb.Game = {}));
})(ThavalonWeb || (ThavalonWeb = {}));
//# sourceMappingURL=gameConstants.js.map