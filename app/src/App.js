// Import everything needed to use the `useQuery` hook
import { useQuery, gql } from '@apollo/client';
import { useCallback, useEffect, useState } from 'react';
import { getAllOutboxEarned, getAllInboxFees } from "./events"
import { Chart as ChartJS } from 'chart.js/auto'
import { Line }            from 'react-chartjs-2'


const GET_LOCATIONS = gql`
  query GetLocations {
    locations {
      id
      name
      description
      photo
    }
  }
`;

export const options = {
  scales: {
    y: {
      beginAtZero: true,
    },
    xAxes: [{
      type: 'time',
      position: 'bottom',
      time: {
        displayFormats: {'day': 'MM/YY'},
        tooltipFormat: 'DD/MM/YY',
        unit: 'month',
       }
    }]
  },
  
};

const labels = ['January', 'February', 'March', 'April', 'May', 'June', 'July'];

const data = {
  labels: ["Jan", "Feb", "Mar", "Apr", "May", "Jun"],
  datasets: [
    {
      label: "First dataset",
      data: [330, 530, 850, -412, -700, 800],
      fill: false,
      borderColor: "rgba(75,192,192,1)",
    },
    {
      label: "Second dataset",
      data: [33, 25, 35, 51, 54, 76],
      fill: false,
      borderColor: "#742774"
    }
  ]
};

function DisplayLocations() {
  const { loading, error, data } = useQuery(GET_LOCATIONS);


  //const test = getAllOutboxEarned();    
  useEffect(() => {const test = getAllInboxFees()}, []);


  if (loading) return <p>Loading...</p>;
  if (error) return <p>Error :(</p>;

  return data.locations.map(({ id, name, description, photo }) => (
    <div key={id}>
      <h3>{name}</h3>
      <img width="400" height="250" alt="location-reference" src={`${photo}`} />
      <br />
      <b>About this location:</b>
      <p>{description}</p>
      <br />
    </div>
  ));
}

export default function App() {

  return (
    <div>
      <Line options={options} data={data} />
      <h2>My first Apollo app 🚀</h2>
      <br/>
      <DisplayLocations />
    </div>
  );
}
