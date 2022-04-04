import { Timeline, Box } from '@primer/react'
import styled from "styled-components"
import { Title, Subtitle, Label, ValueLabel, MapTable, FtTable, NftTable, VarEvent, MapEvent, NftEvent, FtEvent, Controls } from '.';
import { StateExplorerStateUpdateWatchData } from '../../states/NetworkingState';

export const Container = styled.div`
    min-height: 1000px;
    min-width: 800px;

`

const Body = (props: { field?: StateExplorerStateUpdateWatchData }) => {

    if (props.field === undefined) {
        return (<div />);
    }

    let title = props.field.field_name;

    let subtitle = "";
    let value = undefined;
    let events = [];
    if ("Var" in props.field.field_values) {
        subtitle = `Data variable of type ${JSON.stringify(props.field.field_values.Var.value_type)}`;
        value = <ValueLabel name={props.field.field_values.Var.value} />;
        // Events
        let key = 0;
        for (let event of props.field.field_values.Var.events) {
            events.push((<VarEvent key={key += 1} event={event} />))
        }
    } else if ("Map" in props.field.field_values) {
        let entriesCount = props.field.field_values.Map.entries.length;
        let formattedCount = entriesCount === 0 ? "empty" : `${entriesCount} entries`;
        subtitle = `Data map: ${formattedCount}`;
        let entries = [];
        for (let entry of props.field.field_values.Map.entries) {
            entries.push(entry[0]);
        }
        value = <MapTable keyType={props.field.field_values.Map.key_type} valueType={props.field.field_values.Map.value_type} entries={entries} />
        // Events
        let key = 0;
        for (let event of props.field.field_values.Map.events) {
            events.push((<MapEvent key={key += 1} event={event} />))
        }
    } else if ("Nft" in props.field.field_values) {
        let tokensCount = props.field.field_values.Nft.tokens.length;
        let formattedCount = tokensCount === 0 ? "empty" : `${tokensCount} tokens minted`;
        subtitle = `Non fungible token map: ${formattedCount}`;
        let tokens = [];
        for (let entry of props.field.field_values.Nft.tokens) {
            tokens.push(entry[0]);
        }
        value = <NftTable assetType={props.field.field_values.Nft.token_type} tokens={tokens} />
        // Events
        let key = 0;
        for (let event of props.field.field_values.Nft.events) {
            events.push((<NftEvent key={key += 1} event={event} />))
        }
    } else if ("Ft" in props.field.field_values) {
        let balancesCount = props.field.field_values.Ft.balances.length;
        let formattedCount = balancesCount === 0 ? "empty" : `${balancesCount} holders`;
        subtitle = `Fungible token map: ${formattedCount}`;
        let balances = [];
        for (let entry of props.field.field_values.Ft.balances) {
            balances.push(entry[0]);
        }
        value = <FtTable balances={balances} />
        // Events
        let key = 0;
        for (let event of props.field.field_values.Ft.events) {
            events.push((<FtEvent key={key += 1} event={event} />))
        }
    }

    let fieldIdentifier = `${props.field.contract_identifier}::${props.field.field_name}`

    return (
        <Container>
            <Box display="flex" justifyContent="space-between">
                <Title name={title} />
                <Controls fieldIdentifier={fieldIdentifier} />
            </Box>
            <Subtitle name={subtitle} />
            {value}
            <Label name="Latest events" />
            <Timeline>
                {events}
            </Timeline>
        </Container>
    );
};

export { Body };