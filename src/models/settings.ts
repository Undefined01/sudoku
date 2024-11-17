export interface Settings {
  appearance: {
    sudoku: {
      sudokuSvgMargin: number;
      cellSize: number;
      cellStrokeWidth: number;
      boxStrokeWidth: number;

      valueFontSize: number;
      pencilMarkFontSize: number;
      pencilMarkOffset: {
        x: number;
        y: number;
      };
      pencilMarkOffsetWhenHavingCandidates: {
        x: number;
        y: number;
      };
      candidateFontSize: number;

      selectedHighlight: {
        enable: boolean;
        color: string;
        width: number;
      };

      useCellphoneLayoutForPadNumbers: boolean;
      backgroundColorPalette: Array<string>;
    };
  };
}

export const defaultSettings: Settings = {
  appearance: {
    sudoku: {
      sudokuSvgMargin: 10,
      cellSize: 100,
      cellStrokeWidth: 1,
      boxStrokeWidth: 5,

      valueFontSize: 60,
      pencilMarkFontSize: 20,
      pencilMarkOffset: {
        x: 30,
        y: 30,
      },
      pencilMarkOffsetWhenHavingCandidates: {
        x: 15,
        y: 18,
      },
      candidateFontSize: 25,

      selectedHighlight: {
        enable: true,
        color: "blue",
        width: 12,
      },

      useCellphoneLayoutForPadNumbers: true,
      backgroundColorPalette: [
        "#D0D0D0",
        "#C7A89F",
        "#A7B3D6",
        "#F6D6D6",
        "#F6F7C4",
        "#A1EEBD",
        "#7BD3EA",
        "#B55489",
        "#3FC0A0",
      ],
    },
  },
};
