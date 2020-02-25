import React from 'react';
import styled from '@emotion/styled'
import './static/styles.scss';

import { MissionIndicator } from './components/missionIndicators';
import { Card, MissionResult } from './Core/gameConstants';

const AppContainer = styled.div`
  margin: 0;
`;

function App() {
  return (
    <AppContainer>
      <MissionIndicator
        missionNum={1}
        cardsPlayed={[Card.Success, Card.Success, Card.Success]}
        result={MissionResult.Pass}
        playersOnMission={["Paul", "Meg", "Lucas"]}
      />
    </AppContainer>
  );
}

export default App;
