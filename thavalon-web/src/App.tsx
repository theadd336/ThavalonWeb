import React from 'react';
import styled from '@emotion/styled'
import './static/styles.scss';

import {MissionIndicatorCollection } from './components/missionIndicators';
import { Card, MissionResult, Team } from './Core/gameConstants';
import { RoleCaption } from "./components/roleInformation";
const AppContainer = styled.div`
  margin: 0;
`;

function App() {
  return (
    <AppContainer>
      <MissionIndicatorCollection 
        numMissions={5}
        missionsInfo={[
          {
            discriminator: "MissionIndicatorProps",
            missionNum: 1, 
            cardsPlayed: [Card.Success, Card.Success, Card.Success],
            result: MissionResult.Pass,
            playersOnMission: ["Paul", "Meg", "Lucas"]
          }, {
            discriminator: "MissionIndicatorProps",
            missionNum: 2, 
            cardsPlayed: [Card.Success, Card.Success, Card.Success],
            result: MissionResult.Pass,
            playersOnMission: ["Paul", "Meg", "Lucas"]
          }, {
            discriminator: "MissionIndicatorProps",
            missionNum: 3, 
            cardsPlayed: [Card.Success, Card.Success, Card.Success],
            result: MissionResult.Pass,
            playersOnMission: ["Paul", "Meg", "Lucas"]
          }, {
            discriminator: "MissionIndicatorProps",
            missionNum: 4, 
            cardsPlayed: [Card.Success, Card.Success, Card.Success],
            result: MissionResult.Pass,
            playersOnMission: ["Paul", "Meg", "Lucas"]
          }, {
            discriminator: "MissionPlaceholderProps",
            missionNum: 5, 
            numPlayersOnMisison: 5,
            requiresDoubleFail: false
          }]}>
      </MissionIndicatorCollection>
      <br />
      <RoleCaption role="Mordred" team={Team.Evil} />
    </AppContainer>
  );
}

export default App;
