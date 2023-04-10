{
  inputs,
  self,
  ...
}: {
  perSystem = {...}: {
    pre-commit = {
      check.enable = true;

      settings = {
        src = ../.;
        hooks = {
          alejandra.enable = true;
          rustfmt.enable = true;
        };
      };
    };
  };
}
