// Import everything needed to use the `useQuery` hook
import { useQuery, gql } from '@apollo/client';
import { useCallback, useEffect, useState } from 'react';
import { getAllOutboxEarned, getAllInboxFees } from "./events"
import { Chart as ChartJS } from 'chart.js/auto'
import { Line }            from 'react-chartjs-2'
import styled from 'styled-components'
import MyImage from './abacast-logo.png';

const ChartItem = styled.div`
  width: 87%;
  margin-left: auto;
  margin-right: auto;

  canvas{
    border: 10px solid #223345;
    border-radius: 5px;
  }
`

const LogoWrap = styled.div`
  width: 38%;
  margin-left: auto;
  margin-right: auto;

  img{
    width: 100%;
    height: auto;
  }
`

const TextWrap = styled.div`
  font-size: 22px;
`

export const options = {
  scales: {
    y: {
      beginAtZero: true,
    },
    /*xAxis: {
      ticks: {
        beginAtZero: true,
        callback: function(value, index, values) {
          if (index === 0 || index === 1) { 
            return value
          }
        }
      },
      gridLines: {
        display: false
      }
    }*/
  },
};



function Chart() {


/*
{"2022/8/9":-994870926.5,"2022/8/10":-463464129,"2022/8/11":-1202425453,"2022/8/12":-753620417,"2022/8/13":-456948815,"2022/8/14":-655639543,"2022/8/15":-734616709,"2022/8/16":-561250613,"2022/8/17":-907723004,"2022/8/18":-1275292627,"2022/8/19":-807875832,"2022/8/20":-368925630}
*/

  const [chartData, setChartData] = useState({});
  const [loading, setLoading] = useState(true);
  //    
  useEffect(() => {
    (async () => {
      /*const outboxres = await getAllOutboxEarned();
      const inboxres = await getAllInboxFees();
      let netValue = {};
      let gweibase = 1000000000;
      for (var key in outboxres){
        let outboxValueInGwei = outboxres[key] / gweibase // 10^9

        if (inboxres[key]){
          netValue[key] =  outboxValueInGwei - inboxres[key]
        }
      }

      // only show dates where we have data on how much was both sent and receieved... in case contracts changed for inbox gasy fees etc

      console.log("NET VALUE OBJ", netValue, JSON.stringify(netValue));
*/
      let netValue = {"2022/8/9":-994870926.5,"2022/8/10":-463464129,"2022/8/11":-1202425453,"2022/8/12":-753620417,"2022/8/13":-456948815,"2022/8/14":-655639543,"2022/8/15":-734616709,"2022/8/16":-561250613,"2022/8/17":-907723004,"2022/8/18":-1275292627,"2022/8/19":-807875832,"2022/8/20":-368925630}


      let myLabels = [];
      let myValues = [];
      for (var key in netValue){
        myLabels.push(key)
        myValues.push(netValue[key])
      }

      const mydata = {
        labels: myLabels,
        datasets: [
          {
            label: "Relayer net profit/loss (in GWEI)",
            data: myValues,
            fill: false,
            borderColor: "rgba(75,192,192,1)",
          },
        ]
      };


      setChartData(mydata);
      setLoading(false);

    })();
  }, []);


  if (loading) return <p>Loading... this can take a while.</p>;


  return (
    <Line options={options} data={chartData} />
  );
}

export default function App() {

  return (
    <div style={{textAlign:'center'}}>
      <br/><br/>
      <LogoWrap>
        <img src={MyImage} alt="horse" />
      </LogoWrap>
      
      <TextWrap><p>Abacus relayer analytics and incentivization</p></TextWrap>
      
      <br/><br/>
      <ChartItem><Chart></Chart></ChartItem>
      
    </div>
  );
}
