import * as React from 'react';
import { styled } from '@mui/system';
import TablePaginationUnstyled from '@mui/base/TablePaginationUnstyled';
import { ClarityAbiType, isClarityAbiPrimitive, isClarityAbiTuple } from '../../types';

const blue = {
  200: '#A5D8FF',
  400: '#3399FF',
};

const grey = {
  50: '#F3F6F9',
  100: '#E7EBF0',
  200: '#E0E3E7',
  300: '#CDD2D7',
  400: '#B2BAC2',
  500: '#A0AAB4',
  600: '#6F7E8C',
  700: '#3E5060',
  800: '#2D3843',
  900: '#1A2027',
};

const Root = styled('div')(
  ({ theme }) => `
  margin-top: 24px;

  table {
    border-radius: 25px;
    width: 100%;
  }

  td,
  th {
    border: none;
    border-collapse: collapse;
    border-top-style: solid;
    border-top-width: 1px;
    border-top-color: rgb(233, 233, 231);
    color: rgba(55, 53, 47, 0.6);
    font-size: 14px;
    text-align: left;
    padding: 6px;
  }

  td {
    font-size: 14px;
    color: rgb(55, 53, 47);
  }

  `,
);

const CustomTablePagination = styled(TablePaginationUnstyled)(
  ({ theme }) => `
  & .MuiTablePaginationUnstyled-spacer {
    display: none;
  }
  & .MuiTablePaginationUnstyled-toolbar {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;

    @media (min-width: 768px) {
      flex-direction: row;
      align-items: center;
    }
  }
  & .MuiTablePaginationUnstyled-selectLabel {
    margin: 0;
  }
  & .MuiTablePaginationUnstyled-select {
    padding: 2px;
    border: 1px solid ${theme.palette.mode === 'dark' ? grey[800] : grey[200]};
    border-radius: 50px;
    background-color: transparent;
    &:hover {
      background-color: ${theme.palette.mode === 'dark' ? grey[800] : grey[50]};
    }
    &:focus {
      outline: 1px solid ${theme.palette.mode === 'dark' ? blue[400] : blue[200]};
    }
  }
  & .MuiTablePaginationUnstyled-displayedRows {
    margin: 0;

    @media (min-width: 768px) {
      margin-left: auto;
    }
  }
  & .MuiTablePaginationUnstyled-actions {
    padding: 2px;
    border: 1px solid ${theme.palette.mode === 'dark' ? grey[800] : grey[200]};
    border-radius: 50px;
    text-align: center;
  }
  & .MuiTablePaginationUnstyled-actions > button {
    margin: 0 8px;
    border: transparent;
    border-radius: 2px;
    background-color: transparent;
    &:hover {
      background-color: ${theme.palette.mode === 'dark' ? grey[800] : grey[50]};
    }
    &:focus {
      outline: 1px solid ${theme.palette.mode === 'dark' ? blue[400] : blue[200]};
    }
  }
  `,
);

const MapTable = (props: { keyType: ClarityAbiType, valueType: ClarityAbiType, entries: Array<[string, string]> }) => {
  const [page, setPage] = React.useState(0);
  const rowsPerPage = 20;

  // Avoid a layout jump when reaching the last page with empty rows.
  const emptyRows =
    page > 0 ? Math.max(0, (1 + page) * rowsPerPage - props.entries.length) : 0;

  const handleChangePage = (
    event: React.MouseEvent<HTMLButtonElement> | null,
    newPage: number,
  ) => {
    setPage(newPage);
  };

  let keyHeaders = [];
  let rawEntries = props.entries;
  let entries = [];
  if (isClarityAbiTuple(props.keyType)) {
    keyHeaders.push(...props.keyType.tuple.map((key) => key.name));
    for (let entry of rawEntries) {
      let key_values = JSON.parse(entry[0]);
      entries.push(props.keyType.tuple.map((key) => key_values[key.name]));
    }
  } else {
    keyHeaders.push(props.keyType.toString());
    for (let entry of rawEntries) {
      entries.push([entry[0]]);
    }
  } 

  let valueHeaders = [];
  if (isClarityAbiTuple(props.valueType)) {
    valueHeaders.push(...props.valueType.tuple.map((key) => key.name));
    for (let i = 0; i < rawEntries.length; i++) {
      let values = JSON.parse(rawEntries[i][1]);
      entries[i].push(...props.valueType.tuple.map((key) => values[key.name]));
    }
  } else {
    valueHeaders.push(props.valueType.toString());
    for (let i = 0; i < rawEntries.length; i++) {
      entries[i].push(rawEntries[i][1]);
    }
  }
 
  let columnsCount = keyHeaders.length + valueHeaders.length;
  return (
    <Root sx={{ minWidth: 700, width: 800, maxWidth: '100%' }}>
      <table>
        <thead>
          <tr>
            {
              keyHeaders.map((header, i) => {
                return <th style={{color: grey[900]}}>{header}</th>
              })
            }
            {
              valueHeaders.map((header, i) => {
                return <th style={{color: 'rgb(9, 105, 218)'}} >{header}</th>
              })
            }
          </tr>
        </thead>
        <tbody>
          {(rowsPerPage > 0
            ? entries.slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage)
            : entries
          ).map((entry) => (
            <tr key={entry[0]}>
              {entry.map(value => (
                <td>{value}</td>
              ))}
            </tr>
          ))}

          {emptyRows > 0 && (
            <tr style={{ height: 41 * emptyRows }}>
              <td colSpan={columnsCount} />
            </tr>
          )}
        </tbody>
        <tfoot>
          <tr>
            <CustomTablePagination
              colSpan={columnsCount}
              count={props.entries.length}
              rowsPerPage={rowsPerPage}
              page={page}
              componentsProps={{
                actions: {
                  showFirstButton: true,
                  showLastButton: true,
                } as any,
              }}
              onPageChange={handleChangePage}
            />
          </tr>
        </tfoot>
      </table>
    </Root>
  );
}

export { MapTable };